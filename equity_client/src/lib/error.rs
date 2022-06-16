use surf::http::url::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("StdIoError")]
    StdIoError(std::io::Error),
    #[error("BorshDeserializeError")]
    BorshDeserializeError(std::io::Error, Vec<u8>),
    #[error("RonDeserializeError")]
    RonDeserializeError(ron::Error, Vec<u8>),
    #[error("UrlParseError")]
    UrlParseError(#[from] ParseError),
    #[error("SurfError")]
    SurfError(surf::Error),
}

impl From<surf::Error> for Error {
    fn from(e: surf::Error) -> Self {
        Self::SurfError(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
