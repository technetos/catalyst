use bytes::Bytes;
use h2::{self, server::SendResponse, RecvStream};
use serde_json::json;

use futures::{future::Future, stream::Stream};
use std::error::Error;

use catalyst::{
    request::{Json, Request},
    response::Response,
    router::{route_fn, Routes},
    server::start_server,
};

fn greeting(req: Request<Json>) -> Response {
    Response::new()
        .status(http::StatusCode::OK)
        .content_type("application/json")
        .body(Bytes::from(
            serde_json::to_vec(&json!({
                "message": "Greetings earthling!"
            }))
            .unwrap(),
        ))
}

fn main() -> Result<(), Box<std::error::Error>> {
    let mut routes = Routes::new();
    routes.add("/", "GET", greeting);

    start_server(routes)?;
    Ok(())
}
