use crate::{
    error::Error,
    body::Body,
    request::Request,
    response::Response,
    routing_table::RoutingTable,
};
use bytes::Bytes;
use futures::future::{ok as OkFut, Future};
use serde_json::json;
use std::sync::Arc;

pub type RouteF<T> = Box<Future<Item = T, Error = Error> + Send + 'static>;

pub trait Route: Sized + 'static {
    type Body: Body + Send + 'static;
    type Future: Future<Item = Response, Error = Error> + Send + 'static;

    fn handle_request(req: Request<Self::Body>) -> Self::Future;
}

pub(crate) struct Router;

impl Router {
    pub(crate) fn route_request(
        req: Request<h2::RecvStream>,
        table: Arc<RoutingTable>,
    ) -> RouteF<Response> {
        match table.lookup_route(req.parts()) {
            Some(ref route) => route.execute(req),
            _ => Self::not_found(),
        }
    }

    fn not_found() -> RouteF<Response> {
        let res = Response::new()
            .status(http::StatusCode::NOT_FOUND)
            .content_type("application/json")
            .body(json_bytes_ok!(json!({ "message": "not found" })));

        boxed!(OkFut(res))
    }
}
