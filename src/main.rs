use catalyst::router::Router;
use catalyst::server::Server;

use hyper::{self, header, Body, Method, Request, Response, StatusCode};
use serde_json::json;

fn main() {
    let mut router = Router::new();
    router.add_route("/foo", Method::GET, foo);
    router.add_route("/bar", Method::POST, foo);

    let server = Server::new(router);
    server.start();
}

pub fn foo(_req: &Request<Body>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({ "message":"Greetings earthling" })).unwrap(),
        ))
        .unwrap()
}
