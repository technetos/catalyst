use futures::{future::ok as OkFut, future::Future, stream::Stream};
use h2::RecvStream;
use serde_json::json;

pub struct Json(serde_json::Value);

impl Json {
    pub fn inner(&self) -> &serde_json::Value {
        &self.0
    }
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
}

#[derive(Debug)]
pub(crate) enum RequestError {
    SerdeJson(serde_json::Error),
    H2(h2::Error),
}

impl From<h2::Error> for RequestError {
    fn from(e: h2::Error) -> Self {
        RequestError::H2(e)
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(e: serde_json::Error) -> Self {
        RequestError::SerdeJson(e)
    }
}

type RequestFuture<T> = Box<Future<Item = Request<T>, Error = RequestError> + Send + 'static>;

impl Request<Json> {
    pub(crate) fn new(req: http::Request<RecvStream>) -> RequestFuture<Json> {
        let (parts, body) = req.into_parts();

        if &parts.method != http::Method::GET {
            Box::new(receive_json(body).then(|result| {
                Ok(Request::<Json> {
                    parts,
                    body: Json(result?),
                })
            }))
        } else {
            Box::new(OkFut(Request::<Json> {
                parts,
                body: Json(json!({})),
            }))
        }
    }
}

fn receive_json(
    stream: RecvStream,
) -> impl Future<Item = serde_json::Value, Error = RequestError> + Send + 'static {
    stream
        .concat2()
        .then(|result| Ok(serde_json::from_slice(&result?)?))
}
