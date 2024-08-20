use thiserror::Error;

use crate::types::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Emtpy body")]
    EmptyBody,
    #[error("Detached error")]
    DetachedError(String),
    #[error("JSON ser/de error")]
    Json(#[from] serde_json::Error),
    #[error("Telegram error")]
    TelegramError {
        description: String,
        parameters: Option<ResponseParameters>,
    },
}
