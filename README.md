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
## Running the examples
To run an example use  `$ cargo r --example <name_of_example>`

Lets go ahead and run the `simple_routes` example now:
```
$ cargo r --example simple_routes
> starting server on: 127.0.0.1:8000
```

Navigating to `http://localhost:8000/foo` will present you with the message: 
```
{"message":"Greetings earthling"}
```

Congratulations! You're now up and running with Catalyst.

---
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
