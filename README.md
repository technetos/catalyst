# Catalyst

Catalyst is a lightweight, fast and simple to use webserver written on top of hyper.  

## Getting Started

```sh
$ git clone https://github.com/technetos/catalyst.git
```

## Building the project

```sh
$ cargo b
```

## Routes

Functions can be used as routes in Catalyst as long as they match the signature
`(&Request) -> Response`

## Basic Usage

```rust
use catalyst::{router::Router, server::Server};
use hyper::{self, Body, Method, Request, Response};

fn main() {
    // Create a new router.
    let mut router = Router::new();

    // Add routes to the router.
    router.add_route("/foo", Method::GET, foo);

    // Create a new server.
    let server = Server::new(router);

    // Start the server.
    server.start();
}

// Since foo has the signature (&Request<Body>) -> Response<Body> we can use it
// as a route handler.
pub fn foo(req: &Request<Body>) -> Response<Body> {
    println!("{:#?}", req);
    Response::default()
}

```
