use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvsError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("the key `{0}` is not exist")]
    KeyNotFound(String),

    #[error("record have error")]
    RecordError(),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error("failed to exec command, server return error: `{0}`")]
    ServerRespError(String),

    #[error(transparent)]
    SledError(#[from] sled::Error),

    #[error(transparent)]
    RayonBuilderError(#[from] rayon::ThreadPoolBuildError),

    #[error("unknown error")]
    Unknown,
}
