use crate::{
    request::{Json, Request},
    response::Response,
};
use bytes::Bytes;
use futures::future::Future;
use h2::{self, server::SendResponse};
use serde_json::json;
use std::{collections::HashMap, error::Error as StdError};

pub type route_fn = fn(Request<Json>) -> Response;

pub struct Routes {
    routes: HashMap<(String, String), route_fn>,
}

impl Routes {
    pub fn new() -> Self {
        Routes {
            routes: HashMap::new(),
        }
    }

    pub fn add(&mut self, uri: &str, method: &str, fn_ptr: route_fn) {
        self.routes.insert(
            (String::from(uri), String::from(method)),
            fn_ptr as route_fn,
        );
    }
}

pub(crate) struct Router {
    routes: HashMap<(String, String), route_fn>,
}

impl Router {
    pub(crate) fn new(routes: Routes) -> Self {
        Router {
            routes: routes.routes,
        }
    }

    pub(crate) fn handle_request(
        &self,
        req: Request<Json>,
        tx: SendResponse<Bytes>,
    ) -> impl Future<Item = (), Error = ()> + Send + 'static {
        match self.routes.get(&(
            req.parts().uri.path().to_string(),
            req.parts().method.as_str().to_owned(),
        )) {
            Some(ref fn_ptr) => futures::future::ok(send_response(tx, fn_ptr(req))),
            None => {
                let error_res = Response::new()
                    .content_type("application/json")
                    .status(http::StatusCode::NOT_FOUND)
                    .body(json_bytes_ok!(json!({ "error_message":"not found" })));

                futures::future::ok(send_response(tx, error_res))
            }
        }
    }
}

pub(crate) fn send_response(tx: SendResponse<Bytes>, res: Response) {
    if let Err(e) = respond(tx, res) {
        println!("! error: {:?}", e);
    }
}

fn respond(mut tx: SendResponse<Bytes>, res: Response) -> Result<(), Box<StdError>> {
    let (http_res, body) = res.into_inner()?;
    tx.send_response(http_res, false)?.send_data(body, true)?;
    Ok(())
}
