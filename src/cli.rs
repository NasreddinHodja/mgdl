use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long)]
    download: Option<String>,
}

#[derive(Debug)]
pub struct Args {
    pub download: Option<String>,
}

pub fn parse() -> Args {
    let cli = CliArgs::parse();

    Args {
        download: cli.download,
    }
}
