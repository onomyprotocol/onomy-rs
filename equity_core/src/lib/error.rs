#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("StdIoError")]
    StdIoError(#[from] std::io::Error),
    #[error("AddrParseError")]
    AddrParseError(#[from] std::net::AddrParseError),
}

pub type Result<T> = std::result::Result<T, Error>;
