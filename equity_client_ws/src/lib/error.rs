#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("StdIoError")]
    StdIoError(std::io::Error),
    #[error("BorshDeserializeError")]
    SerdeDeserializeError(serde_json::Error, Vec<u8>),
}


pub type Result<T> = std::result::Result<T, Error>;
