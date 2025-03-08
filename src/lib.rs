pub mod cli;
pub mod config;
pub mod db;
pub mod downloader;
pub mod error;
pub mod models;
pub mod scrape;

pub use error::MgdlError;
pub use models::Chapter;
pub use models::Manga;
