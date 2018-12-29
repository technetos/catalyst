use hyper::{header, Body, Method, Request, Response, StatusCode};
use serde_json::json;
use std::collections::HashMap;

/// # RouteHandler
///
/// Any function matching the signature `(&Request<Body>) -> Response<Body>`
/// can be used as a route handler.  
pub trait RouteHandler: Send + Sync + 'static {
    fn respond_to<'r>(&self, req: &'r Request<Body>) -> Response<Body>;
}

impl<F: Send + Sync + 'static> RouteHandler for F
where
    for<'r> F: Fn(&'r Request<Body>) -> Response<Body>,
{
    fn respond_to<'r>(&self, req: &'r Request<Body>) -> Response<Body> {
        self(req)
    }
}

/// # Route
///
/// `Route` represents single route composed of a HTTP method and a handler
/// function.
struct Route {
    /// The HTTP method for the route such as GET, POST, PUT, DELETE, etc.
    method: Method,
    /// The function for this route.
    handler: Box<RouteHandler>,
}

impl Route {
    /// Creates a new route.
    fn new<R: RouteHandler>(method: Method, handler: R) -> Route {
        Route {
            method,
            handler: Box::new(handler),
        }
    }

    /// Returns a reference to the HTTP method for this route.  
    fn method(&self) -> &Method {
        &self.method
    }
}

/// # Router
///
/// The `Router` stores routes as an internal `HashMap<URI, Handler>` where
/// `URI` is the full path to the route and `Handler` is the function called
/// for that route.   
///
/// ### Routing
///
/// If there exists a route matching the URI and HTTP method of the request the
/// `Router` delegates to that route, otherwise the `Router` responds with 404.  
pub struct Router {
    routes: HashMap<String, Route>,
}

impl RouteHandler for Router {
    fn respond_to<'r>(&self, req: &Request<Body>) -> Response<Body> {
        match self.routes.get(req.uri().path()) {
            Some(ref route) if route.method() == req.method() => route.handler.respond_to(req),
            _ => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({ "error_message":"not found" })).unwrap(),
                ))
                .unwrap(),
        }
    }
}

impl Router {
    /// Creates a new `Router`.  
    pub fn new() -> Router {
        Router {
            routes: HashMap::new(),
        }
    }

    /// Adds a route to the router given the `path`, `method` and `handler`.  
    pub fn add_route<R: RouteHandler>(&mut self, path: &str, method: Method, handler: R) {
        self.routes
            .insert(String::from(path), Route::new(method, handler));
    }
}
