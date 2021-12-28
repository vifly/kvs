use serde_json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvStoreError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("the key `{0}` is not exist")]
    KeyNotFound(String),

    #[error("record have error")]
    RecordError(),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error("unknown error")]
    Unknown,
}
