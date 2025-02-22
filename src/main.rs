use mgdl::{cli, config, downloader, MgdlError};

fn main() {
    if let Err(err) = run() {
        eprintln!("Fatal error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), MgdlError> {
    let args = cli::parse();
    let config = config::Config::load()?;
    let dldr = downloader::Downloader::new(config.manga_dir, config.db_dir)?;

    if args.reset {
        println!("Reseting DB...");
        if let Err(err) = dldr.reset_db() {
            eprintln!("{}", err);
        }
    } else if let Some(manga_url) = args.add {
        match dldr.add(&manga_url) {
            Ok(manga) => println!("Added manga {}", &manga.name),
            Err(err) => eprintln!("{}", err),
        };
    } else if let Some(manga_url) = args.download {
        match dldr.download(&manga_url) {
            Ok(manga) => println!("Downloaded manga {}", &manga.name),
            Err(err) => eprintln!("{}", err),
        };
    } else if let Some(manga) = args.update {
        if let Some(manga_name) = manga {
            match manga_name.as_str() {
                "all" => {
                    if let Err(err) = dldr.update_all() {
                        eprintln!("{}", err);
                    };
                }
                manga_name => {
                    match dldr.update(&manga_name) {
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
