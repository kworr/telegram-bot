use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error from Hyper library")]
    Hyper(#[from] hyper::Error),
    #[error("http request error from Hyper library")]
    Http(#[from] hyper::http::Error),
    #[error("Invalid multipart filename")]
    InvalidMultipartFilename,
    #[error("ordinary IO Error")]
    Io(#[from] std::io::Error),
    #[error("raw error from Telegram API")]
    Raw(#[from] telegram_bot_raw::Error),
}
