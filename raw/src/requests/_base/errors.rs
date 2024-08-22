use thiserror::Error;

use crate::types::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Emtpy body")]
    EmptyBody,
    #[error("Detached error: {0}")]
    DetachedError(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("Telegram error:\n{description}\n{parameters:?}")]
    TelegramError {
        description: String,
        parameters: Option<ResponseParameters>,
    },
}
