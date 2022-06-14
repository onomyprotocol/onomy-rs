use surf::http::url::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("StdIoError")]
    StdIoError(#[from] std::io::Error),
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
