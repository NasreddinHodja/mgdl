use clap::{CommandFactory, Parser};

use crate::MgdlError;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Download manga with gallery-dl")]
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

    /// Reset local DB
    #[arg(short, long, default_value_t = false)]
    pub reset: bool,
}

pub fn parse() -> Args {
    let mut args = Args::parse();

    if let Some(None) = args.update {
        args.update = Some(Some("all".to_string()));
    }

    args
}

pub fn print_help() -> Result<(), MgdlError> {
    Args::command().print_help()?;
    Ok(())
}
