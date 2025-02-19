use std::fmt;

#[derive(Debug)]
pub enum MgdlError {
    Config(String),
    Scrape(String),
    Db(String),
}

impl fmt::Display for MgdlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MgdlError::Config(msg) => write!(f, "Config Error: {}", msg),
            MgdlError::Scrape(msg) => write!(f, "Scrape Error: {}", msg),
            MgdlError::Db(msg) => write!(f, "DB Error: {}", msg),
        }
    }
}

impl std::error::Error for MgdlError {}

impl From<std::io::Error> for MgdlError {
    fn from(err: std::io::Error) -> Self {
        MgdlError::Config(err.to_string())
    }
}

impl From<toml::de::Error> for MgdlError {
    fn from(err: toml::de::Error) -> Self {
        MgdlError::Config(err.to_string())
    }
}

impl From<reqwest::Error> for MgdlError {
    fn from(err: reqwest::Error) -> Self {
        MgdlError::Scrape(err.to_string())
    }
}

impl From<rusqlite::Error> for MgdlError {
    fn from(err: rusqlite::Error) -> Self {
        MgdlError::Scrape(err.to_string())
    }
}
