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
`GET` | `https://127.0.0.1:3000/hello-world` |  | ` { "message": "Greetings earthling!" } `
`POST` | `https://127.0.0.1:3000/test-post-json` | _json_ | `true`

## Routes

Functions can be used as routes in Catalyst as long as they match the signature
`(Request<Json>) -> Response`

## Basic Usage

```rust
fn main() -> Result<(), Box<Error>> {
    let mut routes = Routes::new();

    // Add routes to the router.
    routes.add("/hello-world", "GET", greeting);

    // Start the server.
    start_server(routes)?;
    Ok(())
}

fn greeting(req: Request<Json>) -> Response {
    let parts = req.parts();
    println!("Received {} request on path {}", &parts.method, &parts.uri);
    Response::new()
        .status(http::StatusCode::OK)
        .content_type("application/json")
        .body(json_bytes_ok!(json!({ "message": "Greetings earthling!" })))
}
```
