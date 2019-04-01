# Catalyst

[![CircleCI](https://circleci.com/gh/technetos/catalyst/tree/master.svg?style=svg)](https://circleci.com/gh/technetos/catalyst/tree/master)

Catalyst is a lightweight, fast and simple to use webserver written on top of tokio, rustls, and h2.  

## Getting Started

```sh
$ git clone https://github.com/technetos/catalyst.git
```

## Building the project

```sh
$ cargo b --release
```
## Running the examples
To run an example use  `$ cargo r --release --example <name_of_example>`

To run the `simple_routes` example:
```
$ cargo r --release --example simple_routes
Listening on: 127.0.0.1:3000
```
The `simple_routes` example has the following routes

Method | URI | Payload | Expected Response
--- | --- | --- | ---
`GET` | `https://127.0.0.1:3000/index` | JSON | `"hello!"`

## Basic Usage

```rust
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
```
