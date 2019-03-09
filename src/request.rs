use crate::error::Error;
use futures::future::Future;
use h2::RecvStream;

pub trait HttpRequest<T>: Sized + 'static {
    type Future: Future<Item = Self, Error = Error> + Send + 'static;
    fn parse(req: Request<RecvStream>) -> Self::Future;
    fn lift(req: http::Request<RecvStream>) -> Self::Future;
}

pub struct Request<T> {
    parts: http::request::Parts,
    body: T,
}

impl<T> Request<T> {
    pub fn parts(&self) -> &http::request::Parts {
        &self.parts
    }

    pub fn body(&self) -> &T {
        &self.body
    }

    pub fn into_parts(self) -> (http::request::Parts, T) {
        (self.parts, self.body)
    }
}

type RequestF<T> = Box<Future<Item = Request<T>, Error = Error> + Send + 'static>;

impl<T> HttpRequest<T> for Request<T>
where
    T: Body + Send + 'static,
{
    type Future = RequestF<T>;

    fn parse(req: Request<RecvStream>) -> Self::Future {
        let (parts, body) = req.into_parts();

        use http::Method;
        match *&parts.method {
            Method::POST | Method::PUT | Method::PATCH => {
                boxed!(T::parse(body).then(|result| Ok(Request::<T> {
                    parts,
                    body: result?,
                })))
            }
            _ => boxed!(T::default(body).then(|result| Ok(Request::<T> {
                parts,
                body: result?,
            }))),
        }
    }

    fn lift(req: http::Request<RecvStream>) -> Self::Future {
        let (parts, body) = req.into_parts();

        boxed!(T::parse(body).then(|result| Ok(Request::<T> {
            parts,
            body: result?
        })))
    }
}

pub type BodyF<T> = Box<Future<Item = T, Error = Error> + Send + 'static>;

pub trait Body: Sized + 'static {
    type Future: Future<Item = Self, Error = Error> + Send + 'static;

    fn parse(stream: RecvStream) -> Self::Future;
    fn default(stream: RecvStream) -> Self::Future;
}
