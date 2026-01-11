use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Assertion failed: {0}")]
    Assertion(String),

    #[error("Status code {expected} expected, got {actual}")]
    StatusMismatch { expected: u16, actual: u16 },

    #[error("Header '{key}' expected value '{expected}', got '{actual}'")]
    HeaderMismatch {
        key: String,
        expected: String,
        actual: String,
    },

    #[error("Expected JSON response, got content-type: {0}")]
    NotJson(String),

    #[error("JSON path '{path}' not found")]
    PathNotFound { path: String },

    #[error("Field '{field}' expected value {expected}, got {actual}")]
    FieldMismatch {
        field: String,
        expected: String,
        actual: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Interactive prompt error: {0}")]
    Interactive(String),
}

impl From<dialoguer::Error> for Error {
    fn from(err: dialoguer::Error) -> Self {
        Error::Interactive(err.to_string())
    }
}
