use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Http(#[from] hyper::http::Error),
    #[error("Invalid multipart filename")]
    InvalidMultipartFilename,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Raw(#[from] telegram_bot_raw::Error),
}
