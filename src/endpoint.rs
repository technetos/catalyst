use crate::{
    error::Error,
    request::{Body, Request},
    response::Response,
};
use futures::future::Future;

pub trait Endpoint<T>: Sized + 'static
where
    T: Body,
{
    type Future: Future<Item = Response, Error = Error> + Send + 'static;

    fn process_request(
        request: Request<T>,
    ) -> Box<Future<Item = Response, Error = Error> + Send + 'static>;
}

impl<T, R> Endpoint<T> for R
where
    T: Body + Send + 'static,
    R: Route<Body = T> + Send + 'static,
{
    type Future = R::Future;

    fn process_request(
        request: Request<R::Body>,
    ) -> Box<Future<Item = Response, Error = Error> + Send + 'static> {
        boxed!(R::handle_request(request).and_then(|response| Ok(response)))
    }
}

pub type RouteF<T> = Box<Future<Item = T, Error = Error> + Send + 'static>;

pub trait Route: Sized + 'static {
    type Body: Body + Send + 'static;
    type Future: Future<Item = Response, Error = Error> + Send + 'static;

    fn handle_request(req: Request<Self::Body>) -> Self::Future;
}
