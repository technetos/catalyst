use crate::{
    config::Config,
    error::Error,
    request::{HttpRequest, Request},
    response::Response,
    route::Router,
    routing_table::RoutingTable,
};
use bytes::Bytes;
use futures::future::Future;
use futures::stream::Stream;
use h2::{
    server::{handshake, SendResponse},
    RecvStream,
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tokio_rustls::{
    rustls::{
        internal::pemfile::{certs, rsa_private_keys},
        NoClientAuth, ServerConfig, ServerSession,
    },
    TlsAcceptor, TlsStream,
};

pub fn start_server(config: Config, routes: RoutingTable) -> Result<(), Box<std::error::Error>> {
    let tls_cfg = Arc::new(tls_config());
    let table = Arc::new(routes);

    // Parse the arguments into an address.
    let addr = format!("{}:{}", config.address, config.port);
    let addr = addr.parse::<SocketAddr>()?;

    // Bind to a socket (call listen syscall) at `addr`:`port` awaiting new connections.
    let listener = tokio::net::TcpListener::bind(&addr)?;
    println!("Listening on: {}", addr);

    // On an incomming connection, call the accept syscall and construct a
    // new connection between us and the client.
    let server = listener
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |tcp_socket| {
            tokio::spawn({
                let future = setup_tls(tcp_socket, tls_cfg.clone());
                handle_client_requests(future, table.clone())
            });
            Ok(())
        });

    tokio::run(server);
    Ok(())
}

/// Establish an encrypted stream.
fn setup_tls(
    socket: TcpStream,
    cfg: Arc<ServerConfig>,
) -> impl Future<Item = TlsStream<TcpStream, ServerSession>, Error = ()> + Send + 'static {
    TlsAcceptor::from(cfg).accept(socket).map_err(|_| ())
}

fn spawn_request_handler(
    req: http::Request<RecvStream>,
    res: SendResponse<Bytes>,
    table: Arc<RoutingTable>,
) {
    tokio::spawn({
        let routing_table = table.clone();
        let future = Request::<h2::RecvStream>::lift(req);
        let handler = future.and_then(|request| Router::route_request(request, routing_table));

        handler.then(|result| match result {
            Ok(response) => Ok(send_response(res, response)),
            Err(e) => {
                let response = Response::new()
                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .content_type("application/json")
                    .body(json_bytes_ok!(json!({ "error": format!("{:?}", e) })));
                send_response(res, response);
                Ok(())
            }
        })
    });
}

fn handle_client_requests<F>(
    future: F,
    table: Arc<RoutingTable>,
) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    F: Future<Item = TlsStream<TcpStream, ServerSession>, Error = ()> + Send + 'static,
{
    future.and_then(move |tls_socket| {
        let h2_handshake = handshake(tls_socket);

        let dispatch_request = h2_handshake
            .and_then(move |h2_stream| {
                h2_stream.for_each(move |(req, tx)| {
                    spawn_request_handler(req, tx, table.clone());
                    Ok(())
                })
            })
            .and_then(|_| {
                println!("> HTTP/2 Connection Closed");
                Ok(())
            })
            .then(|res| {
                if let Err(e) = res {
                    println!("! {}", e);
                }
                Ok(())
            });
        tokio::spawn(dispatch_request)
    })
}

pub(crate) fn send_response(tx: SendResponse<Bytes>, res: Response) {
    if let Err(e) = respond(tx, res) {
        println!("! error: {:?}", e);
    }

    fn respond(mut tx: SendResponse<Bytes>, res: Response) -> Result<(), Error> {
        let (http_res, body) = res.into_inner()?;
        tx.send_response(http_res, false)?.send_data(body, true)?;
        Ok(())
    }
}

fn tls_config() -> ServerConfig {
    use std::{fs::File, io::BufReader};

    let cert = certs(&mut BufReader::new(
        File::open("ca_cert.pem").expect("Unable to open cert.pem"),
    ))
    .unwrap();
    let mut key = rsa_private_keys(&mut BufReader::new(
        File::open("ca_key.pem").expect("Unable to open key.pem"),
    ))
    .unwrap();

    let mut config = ServerConfig::new(NoClientAuth::new());
    config
        .set_single_cert(cert, key.remove(0))
        .expect("Invalid cert");
    config.set_protocols(&vec!["h2".into()]);

    config
}
