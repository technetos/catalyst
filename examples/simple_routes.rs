use bytes::Bytes;
use futures::future::{ok as OkFut, Future};
use serde_json::json;
use std::error::Error as StdError;

use catalyst::{
    body::{Json, PlainText},
    boxed,
    config::Config,
    endpoint::{Endpoint, Route, RouteF},
    json_bytes_ok,
    request::{HttpRequest, Request},
    response::Response,
    server::start_server,
};
use router_macro::routes;

struct Index;

impl Route for Index {
    type Body = PlainText;
    type Future = RouteF<Response>;

    fn handle_request(req: Request<Self::Body>) -> Self::Future {
        let parts = req.parts();
        println!("Received {} request on path {}", &parts.method, &parts.uri);

        let res = Response::new()
            .status(http::StatusCode::OK)
            .content_type("application/json")
            .body(json_bytes_ok!(json!({ "message": "Greetings earthling!" })));

        boxed!(OkFut(res))
    }
}

struct Profile;

impl Route for Profile {
    type Body = Json;
    type Future = RouteF<Response>;

    fn handle_request(req: Request<Self::Body>) -> Self::Future {
        let parts = req.parts();
        println!("Received {} request on path {}", &parts.method, &parts.uri);

        let res = Response::new()
            .status(http::StatusCode::OK)
            .content_type("application/json")
            .body(json_bytes_ok!(json!(true)));

        boxed!(OkFut(res))
    }
}

#[routes]
struct Router {
    #[get("/")]
    index: Index,
    #[post("/profile")]
    profile: Profile,
}

fn main() -> Result<(), Box<StdError>> {
    // Start the server.
    let config: Config = Config::parse_config()?;
    start_server::<Router>(config)?;
    Ok(())
}
