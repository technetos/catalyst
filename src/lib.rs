#[macro_export]
macro_rules! boxed {
    ($stuff:expr) => {
        Box::new($stuff)
    };
}

#[macro_export]
macro_rules! json_bytes_ok {
    ($json:expr) => {
        Bytes::from(serde_json::to_vec(&$json).unwrap())
    };
}

pub mod body;
pub mod config;
pub mod route;
pub mod error;
pub mod request;
pub mod response;
pub mod server;
pub mod routing_table;
