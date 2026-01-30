use directories::ProjectDirs;
use serde::Deserialize;
use std::{fs, path::PathBuf};

use crate::{utils::expand_tilde, error::{MgdlError, MgdlResult}};

#[derive(Deserialize)]
struct RawConfig {
    manga_dir: String,
}

pub struct Config {
    pub manga_dir: PathBuf,
    pub db_dir: PathBuf,
}

impl Config {
    pub fn load() -> MgdlResult<Self> {
        let project_dirs = ProjectDirs::from("com", "NasreddinHodja", "Mgdl")
            .ok_or_else(|| MgdlError::Config("Could not open config dirs.".to_string()))?;

        let config_dir = project_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        let config_string = fs::read_to_string(config_dir.join("config.toml"))?;
        let raw: RawConfig = toml::from_str(&config_string)?;

        Ok(Self {
            manga_dir: expand_tilde(PathBuf::from(raw.manga_dir))?,
            db_dir: expand_tilde(config_dir.to_path_buf())?,
        })
    }
}
