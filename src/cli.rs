use clap::{CommandFactory, Parser};

use crate::{error::MgdlResult, logger::LogMode, models::ChapterRange};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Download manga rust")]
pub struct Args {
    /// URL of manga to download
    #[arg(short, long)]
    pub download: Option<String>,

    /// URL of manga to add
    #[arg(short, long)]
    pub add: Option<String>,

    /// folder name of manga to update
    #[arg(short, long)]
    pub update: Option<Option<String>>,

    /// URL of manga to scrape
    #[arg(short, long)]
    pub scrape: Option<String>,

    /// chapter range to download (e.g., 5..10, 5.., ..10, 5)
    #[arg(short, long, value_parser = ChapterRange::parse)]
    pub chapters: Option<ChapterRange>,

    /// force redownload of existing pages
    #[arg(short, long, default_value_t = false)]
    pub force: bool,

    /// reset local DB
    #[arg(short, long, default_value_t = false)]
    pub reset: bool,

    /// check all chapters for missing pages and download them
    #[arg(short = 'o', long, default_value_t = false)]
    pub consolidate: bool,

    /// logging mode: plain, fancy, or quiet
    #[arg(short, long, value_enum, default_value = "plain")]
    pub log: LogMode,

    /// verbose output (show INFO messages)
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// print benchmark timing data after download
    #[arg(short, long, default_value_t = false)]
    pub bench: bool,
}

pub fn parse() -> Args {
    let mut args = Args::parse();

    if let Some(None) = args.update {
        args.update = Some(Some("all".to_string()));
    }

    args
}

pub fn print_help() -> MgdlResult<()> {
    Args::command().print_help()?;
    Ok(())
}
