use crate::{
    request::{Json, Request, RequestError},
    response::Response,
    router::{send_response, Router, Routes},
};
use bytes::Bytes;
use futures::{future::Future, stream::Stream};
use h2::server::{handshake, SendResponse};
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

pub fn start_server(routes: Routes) -> Result<(), Box<std::error::Error>> {
    let tls_cfg = Arc::new(tls_config());

    // Parse the arguments into an address.
    let addr = format!("{}:{}", "127.0.0.1", "3000");
    let addr = addr.parse::<SocketAddr>()?;

    let router = Arc::new(Router::new(routes));

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
                setup_http2(future, router.clone())
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
    // TODO: Handle tls errors
    TlsAcceptor::from(cfg).accept(socket).map_err(|e| ())
}

/// Establish a http2 connection.
fn setup_http2<F>(
    future: F,
    router: Arc<Router>,
) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    F: Future<Item = TlsStream<TcpStream, ServerSession>, Error = ()> + Send + 'static,
{
    future.and_then(move |tls_socket| {
        handshake(tls_socket)
            .and_then(move |h2_stream| {
                h2_stream.for_each(move |(req, tx)| {
                    // TODO: Add threadpool of workers to consume multiple requests at a time from the
                    // stream
                    let future = Request::<Json>::new(req);
                    handle_request(future, tx, router.clone()).then(|_| Ok(()))
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
            })
    })
}

/// Dispatch a request to a route handler or report an error in processing the request.
fn handle_request<F>(
    future: F,
    tx: SendResponse<Bytes>,
    router: Arc<Router>,
) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    F: Future<Item = Request<Json>, Error = RequestError> + Send + 'static,
{
    future.then(move |result| match result {
        Ok(request) => {
            router.handle_request(request, tx);
            Ok(())
        }
        Err(e) => {
            let response = Response::new()
                .status(http::StatusCode::BAD_REQUEST)
                .content_type("application/json")
                .body(json_bytes_ok!(json!({ "error": format!("{:?}", e) })));
            send_response(tx, response);
            Ok(())
        }
    })
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
