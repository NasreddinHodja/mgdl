use std::fmt;

#[derive(Debug)]
pub enum MgdlError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    Reqwest(reqwest::Error),
    Rusqlite(rusqlite::Error),
    Parse(std::num::ParseIntError),
    Config(String),
    Db(String),
    Scrape(String),
    Downloader(String),
    Join(tokio::task::JoinError),
}

impl fmt::Display for MgdlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MgdlError::Io(err) => write!(f, "Io error: {}", err),
            MgdlError::Toml(err) => write!(f, "Toml error: {}", err),
            MgdlError::Reqwest(err) => write!(f, "Reqwest error: {}", err),
            MgdlError::Rusqlite(err) => write!(f, "Reqwest error: {}", err),
            MgdlError::Parse(err) => write!(f, "Parse error: {}", err),
            MgdlError::Config(msg) => write!(f, "Config error: {}", msg),
            MgdlError::Scrape(msg) => write!(f, "Scrape error: {}", msg),
            MgdlError::Db(msg) => write!(f, "DB error: {}", msg),
            MgdlError::Downloader(msg) => write!(f, "Downloader error: {}", msg),
            MgdlError::Join(err) => write!(f, "Join error: {}", err),
        }
    }
}

impl std::error::Error for MgdlError {}

impl From<std::io::Error> for MgdlError {
    fn from(err: std::io::Error) -> Self {
        MgdlError::Io(err)
    }
}

impl From<toml::de::Error> for MgdlError {
    fn from(err: toml::de::Error) -> Self {
        MgdlError::Toml(err)
    }
}

impl From<reqwest::Error> for MgdlError {
    fn from(err: reqwest::Error) -> Self {
        MgdlError::Reqwest(err)
    }
}

impl From<rusqlite::Error> for MgdlError {
    fn from(err: rusqlite::Error) -> Self {
        MgdlError::Rusqlite(err)
    }
}

impl From<std::num::ParseIntError> for MgdlError {
    fn from(err: std::num::ParseIntError) -> Self {
        MgdlError::Parse(err)
    }
}

impl From<tokio::task::JoinError> for MgdlError {
    fn from(err: tokio::task::JoinError) -> Self {
        MgdlError::Join(err)
    }
}
