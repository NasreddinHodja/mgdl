use directories::ProjectDirs;
use serde::Deserialize;
use std::{fs, path::PathBuf};

use crate::{
    error::{MgdlError, MgdlResult},
    utils::expand_tilde,
};

#[derive(Deserialize)]
struct RawConfig {
    manga_dir: String,
    base_url: String,
}

pub struct Config {
    pub manga_dir: PathBuf,
    pub db_dir: PathBuf,
    pub base_url: String,
}

impl Config {
    pub fn load() -> MgdlResult<Self> {
        let project_dirs = ProjectDirs::from("com", "NasreddinHodja", "Mgdl")
            .ok_or_else(|| MgdlError::Config("Could not open config dirs.".to_string()))?;

        let config_dir = project_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        let config_path = config_dir.join("config.toml");
        let config_string = fs::read_to_string(&config_path)
            .map_err(|e| MgdlError::Config(format!("{}: {}", config_path.display(), e)))?;
        let raw: RawConfig = toml::from_str(&config_string)?;

        Ok(Self {
            manga_dir: expand_tilde(PathBuf::from(raw.manga_dir))?,
            db_dir: expand_tilde(config_dir.to_path_buf())?,
            base_url: raw.base_url,
        })
    }
}
