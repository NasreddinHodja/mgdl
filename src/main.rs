use mgdl::{cli, config, downloader};

fn main() {
    let args = cli::parse();
    let config = config::Config::load().unwrap();
    let dldr = downloader::Downloader::new(config.manga_dir, config.db_dir);

    if let Some(manga_url) = args.download {
        dldr.download(&manga_url);
    }
}
