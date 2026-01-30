mod cli;
mod config;
mod db;
mod downloader;
mod error;
mod logger;
mod models;
mod scrape;
mod utils;

use error::MgdlError;
use error::MgdlResult;
use models::Chapter;
use models::Manga;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Fatal error: {}", err);
        std::process::exit(1);
    }
}

async fn run() -> MgdlResult<()> {
    let args = cli::parse();
    let config = config::Config::load()?;
    let dldr = downloader::Downloader::new(config.manga_dir, config.db_dir, args.log)?;

    if args.reset {
        if let Err(err) = dldr.reset_db() {
            eprintln!("{}", err);
        }
    } else if let Some(manga_url) = args.add {
        if let Err(err) = dldr.add(&manga_url).await {
            eprintln!("{}", err)
        }
    } else if let Some(manga_url) = args.download {
        if let Err(err) = dldr.download_manga(&manga_url).await {
            eprintln!("{}", err);
        }
    } else if let Some(manga) = args.update {
        if let Some(manga_name) = manga {
            match manga_name.as_str() {
                "all" => {
                    if let Err(err) = dldr.update_all().await {
                        eprintln!("{}", err);
                    };
                }
                manga_name => {
                    if let Err(err) = dldr.update(&manga_name).await {
                        eprintln!("{}", err);
                    };
                }
            };
        }
    } else if let Some(manga_url) = args.scrape {
        if let Err(err) = scrape::scrape_to_csv(&manga_url, None).await {
            eprintln!("{}", err);
        }
    } else {
        if let Err(err) = cli::print_help() {
            eprintln!("{}", err);
        }
    }

    Ok(())
}
