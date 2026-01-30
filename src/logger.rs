use clap::ValueEnum;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::error::{MgdlError, MgdlResult};

#[derive(Clone, Copy, ValueEnum, Debug, PartialEq)]
pub enum LogMode {
    Quiet,
    Plain,
    Fancy,
}

pub struct Logger {
    mode: LogMode,
    multi: Option<MultiProgress>,
}

impl Logger {
    pub fn new(mode: LogMode) -> Self {
        let multi = match mode {
            LogMode::Fancy => Some(MultiProgress::new()),
            _ => None,
        };
        Self { mode, multi }
    }

    pub fn add_spinner(&self, msg: Option<String>) -> MgdlResult<MaybeSpinner> {
        let inner = match self.mode {
            LogMode::Fancy => {
                let spinner = new_spinner()?;
                let spinner = match &self.multi {
                    Some(mp) => mp.add(spinner),
                    None => spinner,
                };
                if let Some(ref msg) = msg {
                    spinner.set_message(msg.clone());
                }
                Some(spinner)
            }
            LogMode::Plain => {
                if let Some(ref msg) = msg {
                    println!("[INFO] {msg}");
                }
                None
            }
            LogMode::Quiet => None,
        };

        Ok(MaybeSpinner {
            inner,
            mode: self.mode,
        })
    }

    pub fn finish_spinner(&self, spinner: MaybeSpinner) {
        if let Some(bar) = spinner.inner {
            if let Some(ref mp) = self.multi {
                bar.finish_and_clear();
                mp.remove(&bar);
            }
        }
    }

    pub fn add_bar(&self, size: u64) -> MgdlResult<MaybeBar> {
        let inner = match self.mode {
            LogMode::Fancy => {
                let bar = new_progress_bar(size)?;
                Some(match &self.multi {
                    Some(mp) => mp.add(bar),
                    None => bar,
                })
            }
            _ => None,
        };

        Ok(MaybeBar {
            inner,
            mode: self.mode,
        })
    }

    pub fn finish_bar(&self, bar: MaybeBar) {
        if let Some(pb) = bar.inner {
            if let Some(ref mp) = self.multi {
                pb.finish_and_clear();
                mp.remove(&pb);
            }
        }
    }
}

// -- Null-object wrappers ----------------------------------------------------

pub struct MaybeSpinner {
    inner: Option<ProgressBar>,
    mode: LogMode,
}

impl MaybeSpinner {
    pub fn set_message(&self, msg: String) {
        match self.mode {
            LogMode::Fancy => {
                if let Some(ref s) = self.inner {
                    s.set_message(msg);
                }
            }
            LogMode::Plain => println!("[INFO] {msg}"),
            LogMode::Quiet => {}
        }
    }
}

pub struct MaybeBar {
    inner: Option<ProgressBar>,
    mode: LogMode,
}

impl MaybeBar {
    pub fn set_prefix(&self, msg: String) {
        if let Some(ref bar) = self.inner {
            bar.set_prefix(msg);
        }
    }

    pub fn inc(&self, delta: u64) {
        if let Some(ref bar) = self.inner {
            bar.inc(delta);
        }
    }

    pub fn success(&self, msg: String) {
        match self.mode {
            LogMode::Fancy => {
                if let Some(ref bar) = self.inner {
                    bar.println(format!("[SUCCESS] {msg}"));
                }
            }
            LogMode::Plain => println!("[SUCCESS] {msg}"),
            LogMode::Quiet => {}
        }
    }
}

// -- Progress bar / spinner constructors (private) ---------------------------

fn new_progress_bar(size: u64) -> MgdlResult<ProgressBar> {
    let bar = ProgressBar::new(size);
    let style = ProgressStyle::with_template("{prefix} {elapsed_precise} {wide_bar} {pos}/{len}")
        .map_err(|e| MgdlError::Scrape(e.to_string()))?;
    bar.set_style(style);
    Ok(bar)
}

fn new_spinner() -> MgdlResult<ProgressBar> {
    let spinner = ProgressBar::new_spinner();
    let style = ProgressStyle::with_template("{spinner} {msg}")
        .map_err(|e| MgdlError::Scrape(e.to_string()))?;
    spinner.set_style(style);
    spinner.enable_steady_tick(Duration::from_millis(50));
    Ok(spinner)
}
