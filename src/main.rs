use mgdl::{cli, config, downloader, MgdlError};

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Fatal error: {}", err);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), MgdlError> {
    let args = cli::parse();
    let config = config::Config::load()?;
    let dldr = downloader::Downloader::new(config.manga_dir, config.db_dir)?;

    if args.reset {
        println!("Reseting DB...");
        if let Err(err) = dldr.reset_db() {
            eprintln!("{}", err);
        }
    } else if let Some(manga_url) = args.add {
        match dldr.add(&manga_url).await {
            Ok((manga, _chapters)) => println!("Added manga {}", &manga.name),
            Err(err) => eprintln!("{}", err),
        };
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
                    match dldr.update(&manga_name).await {
                        Ok(manga) => println!("Updated {}", manga.name),
                        Err(err) => eprintln!("{}", err),
                    };
                }
            };
        }
    } else {
        if let Err(err) = cli::print_help() {
            eprintln!("{}", err);
        }
    }

    Ok(())
}
