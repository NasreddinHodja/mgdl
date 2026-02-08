#[cfg(feature = "bench")]
mod bench;
mod cli;
mod config;
mod db;
mod downloader;
mod error;
mod logger;
mod models;
mod scrape;
mod utils;

use std::time::Duration;

use error::MgdlResult;

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
    let base_url = config.base_url;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    #[cfg(feature = "bench")]
    let bench = if args.bench {
        Some(bench::BenchCollector::new())
    } else {
        None
    };

    #[cfg(feature = "bench")]
    let config_dir = config.db_dir.clone();

    let dldr = downloader::Downloader::new(
        config.manga_dir,
        config.db_dir,
        base_url.clone(),
        args.log,
        args.verbose,
        client.clone(),
        #[cfg(feature = "bench")]
        bench.clone(),
    )?;

    if args.reset {
        dldr.reset_db()?;
    } else if args.consolidate {
        dldr.consolidate_all().await?;
    } else if let Some(manga_url) = args.add {
        dldr.add(&manga_url).await?;
    } else if let Some(manga_url) = args.download {
        let manga = dldr
            .download_manga(&manga_url, args.chapters.as_ref(), args.force)
            .await?;
        #[cfg(feature = "bench")]
        if let Some(bench) = bench {
            let report = bench.finish(&manga.name);
            report.print_summary();
            report.write_json(&config_dir);
        }
        let _ = manga;
    } else if let Some(manga) = args.update {
        if let Some(manga_name) = manga {
            match manga_name.as_str() {
                "all" => dldr.update_all().await?,
                manga_name => {
                    dldr.update(manga_name).await?;
                }
            };
        }
    } else if let Some(manga_url) = args.scrape {
        #[cfg(feature = "bench")]
        scrape::scrape_to_csv(&client, &base_url, &manga_url, None).await?;
        #[cfg(not(feature = "bench"))]
        {
            let _ = manga_url;
            eprintln!("scrape-to-csv requires the 'bench' feature: cargo run --features bench -- -s <URL>");
        }
    } else {
        cli::print_help()?;
    }

    Ok(())
}
