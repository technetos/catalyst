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
`GET` | `https://127.0.0.1:3000/` |  | ` { "message": "Greetings earthling!" } `
`POST` | `https://127.0.0.1:3000/profile` | JSON | `true`

## Basic Usage

```rust
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

#[routes]
struct Router {
    #[get("/")]
    index: Index,
}

fn main() -> Result<(), Box<StdError>> {
    // Start the server.
    let config: Config = Config::parse_config()?;
    start_server::<Router>(config)?;
    Ok(())
}
```
