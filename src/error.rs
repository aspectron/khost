use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),

    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    // #[error(transparent)]
    // Http(#[from] workflow_http::error::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Encryption(#[from] workflow_encryption::error::Error),

    #[error(transparent)]
    Utils(#[from] workflow_utils::error::Error),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Not found")]
    NotFound,

    // #[error(transparent)]
    // ConfigBuilder(#[from] crate::systemd::ConfigBuilderError),
    #[error("Root privilege required (run `sudo khost`)")]
    Sudo,
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Custom(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Custom(s.to_string())
    }
}

impl Error {
    pub fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}
