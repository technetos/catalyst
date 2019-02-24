use bytes::Bytes;
use serde_json::json;

use std::error::Error;

use catalyst::{
    config::Config,
    json_bytes_ok,
    request::{Json, Request},
    response::Response,
    router::Routes,
    server::start_server,
};

fn greeting(req: Request<Json>) -> Response {
    let parts = req.parts();
    println!("Received {} request on path {}", &parts.method, &parts.uri);
    Response::new()
        .status(http::StatusCode::OK)
        .content_type("application/json")
        .body(json_bytes_ok!(json!({ "message": "Greetings earthling!" })))
}

fn post_json(req: Request<Json>) -> Response {
    let parts = req.parts();
    println!("Received {} request on path {}", &parts.method, &parts.uri);
    println!("Body JSON: {}", req.body().inner());
    Response::new()
        .status(http::StatusCode::OK)
        .content_type("application/json")
        .body(json_bytes_ok!(json!(true)))
}

fn main() -> Result<(), Box<Error>> {
    let mut routes = Routes::new();
    let config: Config = Config::parse_config()?;

    // Add routes to the router.
    routes.add("/hello-world", "GET", greeting);
    routes.add("/test-post-json", "POST", post_json);

    // Start the server.
    start_server(config, routes)?;
    Ok(())
}
