use dirs::home_dir;
use std::fs;
use std::path::PathBuf;
use toml;

use directories::ProjectDirs;
use serde::Deserialize;

use crate::error::MgdlError;

pub type Result<T> = std::result::Result<T, MgdlError>;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub manga_dir: String,
}

pub struct MgdlConfig {
    pub manga_dir: PathBuf,
    pub db_dir: PathBuf,
}

impl Config {
    pub fn load() -> Result<MgdlConfig> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "NasreddinHodja", "Mgdl") {
            let config_dir = proj_dirs.config_dir();

            fs::create_dir_all(config_dir)?;

            let config_file = config_dir.join("config.toml");
            let config_string = fs::read_to_string(&config_file)?;
            let config: Config = toml::from_str(&config_string)?;
            let mgdl_config = MgdlConfig {
                manga_dir: expand_tilde(PathBuf::from(&config.manga_dir)),
                db_dir: expand_tilde(PathBuf::from(&config_dir.to_str().unwrap().to_string())),
            };

            Ok(mgdl_config)
        } else {
            Err(MgdlError::Config("Could not open config dirs.".to_string()))
        }
    }
}

fn expand_tilde(path: PathBuf) -> PathBuf {
    if let Some(home) = home_dir() {
        if path.starts_with("~") {
            return home.join(path.strip_prefix("~").unwrap());
        }
    }
    path
}
