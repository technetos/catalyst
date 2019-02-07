use bytes::Bytes;

pub struct Response {
    res: http::response::Builder,
    data: Bytes,
}

impl Response {
    pub(crate) fn into_inner(
        mut self,
    ) -> Result<(http::Response<()>, Bytes), Box<std::error::Error>> {
        let response = self.res.body(())?;
        Ok((response, self.data))
    }

    pub fn new() -> Response {
        Response {
            res: http::Response::builder(),
            data: Bytes::default(),
        }
    }

    pub fn status(mut self, status: http::StatusCode) -> Response {
        self.res.status(status);
        self
    }

    pub fn content_type(mut self, content_ty: &str) -> Response {
        self.res.header("Content-Type", content_ty);
        self
    }

    pub fn body(mut self, body: Bytes) -> Response {
        self.data = body;
        self
    }
}
