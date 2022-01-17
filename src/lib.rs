use serde::{Deserialize, Serialize};

pub use client::KvsClient;
pub use engines::KvsEngine;
pub use kvs_engine::KvStore;
pub use server::KvsServer;
pub use sled_engine::SledKvsEngine;

use crate::error::KvsError;

mod error;
mod engines;
mod server;
mod client;
mod kvs_engine;
mod sled_engine;

pub type Result<T> = std::result::Result<T, KvsError>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    is_ok: bool,
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

impl Response {
    pub fn new(is_ok: bool, data: String) -> Response {
        Response { is_ok, data }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
