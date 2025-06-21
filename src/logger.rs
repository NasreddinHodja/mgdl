use clap::ValueEnum;
use indicatif::MultiProgress;

use crate::{
    maybe_progress::{MaybeBar, MaybeSpinner},
    MgdlResult,
};

#[derive(Clone, Copy, ValueEnum, Debug, PartialEq)]
pub enum LogMode {
    Quiet,
    Plain,
    Normal,
}

const DEFAULT_MODE: LogMode = LogMode::Normal;

pub struct Logger {
    mode: LogMode,
    multi_progress: Option<MultiProgress>,
}

impl Logger {
    pub fn new(mode: Option<LogMode>) -> Self {
        let mode = match mode {
            Some(mode) => mode,
            None => DEFAULT_MODE,
        };

        let multi_progress = Some(MultiProgress::new());

        Self {
            mode,
            multi_progress,
        }
    }

    pub fn add_spinner(&self, msg: Option<String>) -> MgdlResult<MaybeSpinner> {
        let spinner = MaybeSpinner::new(self.multi_progress.as_ref(), msg, self.mode)?;
        return Ok(spinner);
    }

    pub fn finish_spinner(&self, spinner: MaybeSpinner) {
        spinner.finish_and_clear(self.multi_progress.as_ref());
    }

    pub fn add_bar(&self, size: u64) -> MgdlResult<MaybeBar> {
        let bar = MaybeBar::new(self.multi_progress.as_ref(), size, self.mode)?;
        return Ok(bar);
    }

    pub fn finish_bar(&self, bar: MaybeBar) {
        bar.finish_and_clear(self.multi_progress.as_ref());
    }
}
