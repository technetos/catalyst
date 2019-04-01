use crate::{request::{HttpRequest, Request}, route::{Route, RouteF}, response::Response};
use futures::future::Future;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct RoutingTableEntry {
    // A closure that returns the route handler to be evaluated
    lazy: fn(Request<h2::RecvStream>) -> RouteF<Response>,
}

impl RoutingTableEntry {
    pub(crate) fn execute(&self, req: Request<h2::RecvStream>) -> RouteF<Response> {
        (self.lazy)(req)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct RoutingTableKey {
    pub method: http::Method,
    pub dynamic: bool,
    pub segment: String,
}

#[derive(Debug)]
pub struct RoutingTable {
    route: Option<RoutingTableEntry>,
    hash_map: HashMap<RoutingTableKey, RoutingTable>,
    params: HashMap<String, usize>,
}

impl RoutingTable {
    pub fn new() -> RoutingTable {
        Self {
            route: None,
            hash_map: HashMap::new(),
            params: HashMap::new(),
        }
    }

    pub(crate) fn lookup_route(&self, parts: &http::request::Parts) -> Option<&RoutingTableEntry> {
        let mut table = self;
        let path = &parts.uri.path();
        let method = &parts.method;

        let dynamic_key = RoutingTableKey {
            method: method.clone(),
            dynamic: true,
            segment: String::new(),
        };

        for segment in path.split("/") {
            if segment.is_empty() {
                continue;
            }
            // Inspect path for dynamic segment
            if let Some(next) = table.hash_map.get(&dynamic_key) {
                table = next;
            // If no dynamic segments are found, search for the exact segment
            } else if let Some(next) = table.hash_map.get(&RoutingTableKey {
                method: method.clone(),
                dynamic: false,
                segment: segment.into(),
            }) {
                table = next;
            // Otherwise the route doesnt exist
            } else {
                return None;
            }
        }

        table.route.as_ref()
    }

    pub fn attach<E>(&mut self, _route: E)
    where
        E: Route,
    {
        let entry = RoutingTableEntry {
            lazy: |req: Request<h2::RecvStream>| -> RouteF<Response> {
                boxed!(Request::<E::Body>::parse(req).and_then(|request| E::handle_request(request)))
            },
        };

        self.route = Some(entry);
    }

    pub fn at(&mut self, path: &str, method: http::Method) -> &mut RoutingTable {
        let mut params = HashMap::new();
        if path == "/" {
            panic!("Illegal path");
        }

        let mut table = self;

        let dynamic_key = RoutingTableKey {
            method: method.clone(),
            dynamic: true,
            segment: String::new(),
        };

        for (i, segment) in path.split("/").enumerate() {
            if segment.is_empty() {
                continue;
            }
            if segment.starts_with("{") && segment.ends_with("}") {
                params.insert(segment[1..segment.len() - 1].into(), i);
                table = table
                    .hash_map
                    .entry(dynamic_key.clone())
                    .or_insert(RoutingTable::new());
            } else {
                table = table
                    .hash_map
                    .entry(RoutingTableKey {
                        method: method.clone(),
                        dynamic: false,
                        segment: segment.to_string(),
                    })
                    .or_insert(RoutingTable::new());
            }
        }
        table.params = params;
        table
    }
}
