use bytes::Bytes;
use futures::future::{ok as OkFut, Future};
use serde_json::json;
use std::error::Error as StdError;

use catalyst::{
    body::{Json, PlainText},
    boxed,
    config::Config,
    json_bytes_ok,
    request::Request,
    response::Response,
    route::{Route, RouteF},
    routing_table::RoutingTable,
    server::start_server,
};

struct HomePage;

impl Route for HomePage {
    type Body = Json;
    type Future = RouteF<Response>;

    fn handle_request(req: Request<Self::Body>) -> Self::Future {
        let parts = req.parts();

        let user_settings = req.body().inner();

        let res = Response::new()
            .status(http::StatusCode::OK)
            .content_type("application/json")
            .body(json_bytes_ok!(json!("hello!")));

        boxed!(OkFut(res))
    }
}

use http::Method;

fn main() -> Result<(), Box<StdError>> {
    // Configure the server.
    let config: Config = Config::parse_config()?;

    // Define the routes.
    let mut routing_table = RoutingTable::new();
    routing_table
        .at("/index", Method::GET)
        .attach(HomePage);
        
    // Start the server.
    start_server(config, routing_table)?;
    Ok(())
}
