use crate::response::Response;
use bytes::Bytes;
use futures::{future::Future, stream::Stream};
use h2::RecvStream;
use serde_json::json;

pub struct Json(serde_json::Value);

impl Json {
    pub fn body(&self) -> &serde_json::Value {
        &self.0
    }
}

pub struct Request<T> {
    parts: http::request::Parts,
    body: T,
}

impl Request<Json> {
    pub(crate) fn new(
        req: http::Request<RecvStream>,
    ) -> Result<Request<Json>, Box<std::error::Error>> {
        let (parts, body) = req.into_parts();

        let mut json = Json(json!({}));
        if &parts.method != http::Method::GET {
            json = Json(serde_json::from_slice(&body.concat2().wait()?)?);
        }

        Ok(Request::<Json> { parts, body: json })
    }
}
