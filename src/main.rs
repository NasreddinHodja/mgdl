use mgdl::{cli, config, downloader};

fn main() {
    let args = cli::parse();
    let config = config::Config::load().unwrap();
    let dldr = downloader::Downloader::new(config.manga_dir, config.db_dir);

    if args.reset {
        println!("Reseting DB...");
        dldr.reset_db();
    } else if let Some(manga_url) = args.add {
        dldr.add(&manga_url);
    } else if let Some(manga_url) = args.download {
        dldr.download(&manga_url);
    } else if let Some(manga_name) = args.update {
        let result = dldr.update(&manga_name);

        if let Ok(manga) = result {
            println!("Updated {}", manga.name);
        } else {
            eprintln!("Couldn't find manga in DB")
        }
    } else {
        cli::print_help();
    }
}
