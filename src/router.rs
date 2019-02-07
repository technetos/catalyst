use crate::{
    request::{Json, Request},
    response::Response,
};
use bytes::Bytes;
use h2::{self, server::SendResponse, RecvStream};
use serde_json::json;
use std::collections::HashMap;

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
        req: http::Request<RecvStream>,
        mut tx: SendResponse<Bytes>,
    ) {
        let mut response = Response::new().content_type("application/json");

        match self.routes.get(&(
            req.uri().path().to_string(),
            req.method().as_str().to_owned(),
        )) {
            Some(ref fn_ptr) => {
                let json_req = Request::<Json>::new(req);
                if let Err(e) = json_req {
                    let error_res =
                        response
                            .status(http::StatusCode::BAD_REQUEST)
                            .body(Bytes::from(
                                serde_json::to_vec(&json!({ "error": format!("{}", e) })).unwrap(),
                            ));

                    send_response(tx, error_res);
                } else {
                    send_response(tx, fn_ptr(json_req.unwrap()));
                }
            }
            None => {
                let error_message = Bytes::from(
                    serde_json::to_vec(&json!({ "error_message":"not found" })).unwrap(),
                );
                let error_res = response
                    .status(http::StatusCode::NOT_FOUND)
                    .body(error_message);

                send_response(tx, error_res);
            }
        }
    }
}

pub(crate) fn send_response(mut tx: SendResponse<Bytes>, res: Response) {
    if let Err(e) = respond(tx, res) {
        println!("! error: {:?}", e);
    }
}

fn respond(mut tx: SendResponse<Bytes>, res: Response) -> Result<(), Box<std::error::Error>> {
    let (http_res, body) = res.into_inner()?;
    tx.send_response(http_res, false)?.send_data(body, true)?;
    Ok(())
}
