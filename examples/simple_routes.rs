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
            .body(json_bytes_ok!(
                json!({ "message": "Welcome to the profile" })
            ));

        boxed!(OkFut(res))
    }
}

pub struct Router;

impl Route for Router {
    type Body = h2::RecvStream;
    type Future = RouteF<Response>;

    fn handle_request(req: Request<Self::Body>) -> Self::Future {
        let parts = &req.parts();
        match parts.uri.path() {
            "/" if parts.method == http::Method::GET => {
                type IndexBody = <Index as Route>::Body;

                boxed!(Request::<IndexBody>::parse(req)
                    .and_then(|request| Index::process_request(request)))
            }
            "/profile" if parts.method == http::Method::POST => {
                type ProfileBody = <Profile as Route>::Body;

                boxed!(Request::<ProfileBody>::parse(req)
                    .and_then(|request| Profile::process_request(request)))
            }
            _ => {
                let res = Response::new()
                    .status(http::StatusCode::NOT_FOUND)
                    .content_type("application/json")
                    .body(json_bytes_ok!(json!({ "message": "not found" })));

                boxed!(OkFut(res))
            }
        }
    }
}

fn main() -> Result<(), Box<StdError>> {
    // Start the server.
    let config: Config = Config::parse_config()?;
    start_server(config, Router)?;
    Ok(())
}
