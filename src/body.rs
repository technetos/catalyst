use crate::request::{Body, BodyF};
use futures::{future::ok as OkFut, future::Future, stream::Stream};
use h2::RecvStream;
use serde_json::json;

pub struct Json(serde_json::Value);

impl Json {
    pub fn inner(&self) -> &serde_json::Value {
        &self.0
    }
}

impl Body for Json {
    type Future = BodyF<Self>;

    fn parse(stream: RecvStream) -> Self::Future {
        boxed!(stream
            .concat2()
            .then(|result| Ok(serde_json::from_slice(&result?)?))
            .and_then(|value: serde_json::Value| Ok(Json(value))))
    }

    fn default(_: RecvStream) -> Self::Future {
        boxed!(OkFut(Json(json!({}))))
    }
}

pub struct PlainText(String);

impl PlainText {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl Body for PlainText {
    type Future = BodyF<Self>;

    fn parse(stream: RecvStream) -> Self::Future {
        boxed!(stream
            .concat2()
            .then(|result| Ok(String::from_utf8(result?.to_vec())?))
            .and_then(|value: String| Ok(PlainText(value))))
    }

    fn default(_: RecvStream) -> Self::Future {
        boxed!(OkFut(PlainText(String::new())))
    }
}

impl Body for h2::RecvStream {
    type Future = BodyF<Self>;

    fn parse(stream: RecvStream) -> Self::Future {
        boxed!(OkFut(stream))
    }

    fn default(stream: RecvStream) -> Self::Future {
        boxed!(OkFut(stream))
    }
}
