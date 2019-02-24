#[macro_export]
macro_rules! json_bytes_ok {
    ($json:expr) => {
        Bytes::from(serde_json::to_vec(&$json).unwrap())
    };
}

pub mod config;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
