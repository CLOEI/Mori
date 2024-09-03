use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Network error: {0}")]
    NetworkError(#[from] ureq::Error),
    #[error("Steam initialization error: {0}")]
    SteamError(String),
    #[error("Other error: {0}")]
    Other(String),
}
