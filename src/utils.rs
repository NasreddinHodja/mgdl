use directories::BaseDirs;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::{path::PathBuf, time::Duration};

use crate::MgdlError;

type Result<T> = std::result::Result<T, MgdlError>;

pub fn normalize(s: &str) -> String {
    let replacements = vec![
        (Regex::new(r"[áàâãä]").unwrap(), "a"),
        (Regex::new(r"[éèêë]").unwrap(), "e"),
        (Regex::new(r"[íìîï]").unwrap(), "i"),
        (Regex::new(r"[óòôõö]").unwrap(), "o"),
        (Regex::new(r"[úùûü]").unwrap(), "u"),
        (Regex::new(r"[ç]").unwrap(), "c"),
        (Regex::new(r"[ñ]").unwrap(), "n"),
        (Regex::new(r"[ýÿ]").unwrap(), "y"),
        (Regex::new(r"[ÁÀÂÃÄ]").unwrap(), "A"),
        (Regex::new(r"[ÉÈÊË]").unwrap(), "E"),
        (Regex::new(r"[ÍÌÎÏ]").unwrap(), "I"),
        (Regex::new(r"[ÓÒÔÕÖ]").unwrap(), "O"),
        (Regex::new(r"[ÚÙÛÜ]").unwrap(), "U"),
        (Regex::new(r"[Ç]").unwrap(), "C"),
        (Regex::new(r"[Ñ]").unwrap(), "N"),
        (Regex::new(r"[Ý]").unwrap(), "Y"),
    ];

    let mut s = s.to_string();

    for (re, replacement) in replacements.iter() {
        s = re.replace_all(&s, *replacement).to_string();
    }

    let re = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
    s = re.replace_all(&s, "_").to_string();

    s.trim_matches('_').to_lowercase()
}

pub fn extract_hash(url: &str) -> Option<String> {
    let re = Regex::new(r"/series/([^/]+)(?:/|$)").ok()?;
    re.captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

pub fn expand_tilde(path: PathBuf) -> Result<PathBuf> {
    if let Some(stripped) = path.strip_prefix("~").ok() {
        if let Some(base_dirs) = BaseDirs::new() {
            return Ok(base_dirs.home_dir().join(stripped));
        } else {
            return Err(MgdlError::Config(
                "Could not determine home directory".to_string(),
            ));
        }
    }

    Ok(path)
}

pub fn gen_progress_bar(size: u64) -> Result<ProgressBar> {
    let bar = ProgressBar::new(size);
    let style =
        ProgressStyle::with_template("{prefix} {elapsed_precise} {wide_bar} {pos}/{len}")
            .map_err(|_| MgdlError::Scrape("Could not create progress bar style".to_string()))?;
    bar.set_style(style);

    Ok(bar)
}

pub fn gen_progress_spinner() -> Result<ProgressBar> {
    let spinner = ProgressBar::new_spinner();
    let style = ProgressStyle::with_template("{spinner} {msg}")
        .map_err(|_| MgdlError::Scrape("Could not create spinner style".to_string()))?;
    spinner.set_style(style);
    spinner.enable_steady_tick(Duration::from_millis(50));

    Ok(spinner)
}

pub fn debug_writeln(line: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("./debug.log")?;

    writeln!(file, "{}", line.to_string())?;

    Ok(())
}
