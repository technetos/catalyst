use crate::router::{route_fn, Router, Routes};
use futures::{future::Future, stream::Stream};
use http::{Request, Response};
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_rustls::{
    rustls::{
        internal::pemfile::{certs, rsa_private_keys},
        Certificate, NoClientAuth, PrivateKey, ServerConfig,
    },
    TlsAcceptor,
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
            tokio::spawn(tls_http2(tcp_socket, router.clone(), tls_cfg.clone()));
            Ok(())
        });
    tokio::run(server);
    Ok(())
}

fn tls_http2(
    socket: TcpStream,
    router: Arc<Router>,
    cfg: Arc<ServerConfig>,
) -> impl Future<Item = (), Error = ()> {
    let tls_acceptor = TlsAcceptor::from(cfg);
    let tls_accept = tls_acceptor
        .accept(socket)
        .map_err(|e| ())
        .and_then(move |tls_socket| {
            // Note, if the client sends a very large payload the target window size must be
            // adjusted.
            let http2_connect = h2::server::handshake(tls_socket)
                // When the HTTP/2 handshake is complete, we have an established stream between us
                // and the client.  This stream is what the client uses to send us http requests.
                .and_then(move |http2_stream| {
                    dbg!("h2 connection bound");
                    http2_stream.for_each(move |(req, res)| {
                        // Since communication between the server and client is acheived using a
                        // stream, if the server blocks while processing a request the client is
                        // NOT blocked.  Ths client is waiting for data to arrive on the stream but
                        // is in no way prevented from continuing execution.
                        //
                        // The client is however blocked with respect to having further requests
                        // processed before the blocking one has completed.  It is possible to fill
                        // the stream to capacity by sending many requests following one that
                        // blocks, at which point no further requests  will be received.
                        router.handle_request(req, res);
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

            tokio::spawn(http2_connect)
        });

    return tls_accept;
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
