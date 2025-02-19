use clap::{CommandFactory, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Download manga with gallery-dl")]
pub struct Args {
    // URL of manga to download
    #[arg(short, long)]
    pub download: Option<String>,

    // URL of manga to add
    #[arg(short, long)]
    pub add: Option<String>,

    // folder name of manga to update
    #[arg(short, long)]
    pub update: Option<String>,

    // Reset database
    #[arg(short, long, default_value_t = false)]
    pub reset: bool,
}

pub fn parse() -> Args {
    let args = Args::parse();

    args
}

pub fn print_help() {
    Args::command().print_help().unwrap();
}
