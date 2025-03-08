use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use toml;

use crate::utils::expand_tilde;
use crate::MgdlError;

pub type Result<T> = std::result::Result<T, MgdlError>;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub manga_dir: String,
}

#[derive(Debug)]
pub struct MgdlConfig {
    pub manga_dir: PathBuf,
    pub db_dir: PathBuf,
}

impl Config {
    pub fn load() -> Result<MgdlConfig> {
        let project_dirs = ProjectDirs::from("com", "NasreddinHodja", "Mgdl")
            .ok_or_else(|| MgdlError::Config("Could not open config dirs.".to_string()))?;

        let config_dir = project_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        let config_file = config_dir.join("config.toml");
        let config_string = fs::read_to_string(&config_file)?;
        let config: Config = toml::from_str(&config_string)?;

        let manga_dir = expand_tilde(PathBuf::from(&config.manga_dir))?;
        let db_dir = expand_tilde(config_dir.to_path_buf())?;

        Ok(MgdlConfig { manga_dir, db_dir })
    }
}
