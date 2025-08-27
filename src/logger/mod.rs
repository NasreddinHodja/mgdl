use crate::MgdlResult;
use clap::ValueEnum;
use indicatif::MultiProgress;

mod maybe_progress;
use maybe_progress::{MaybeBar, MaybeSpinner};

#[derive(Clone, Copy, ValueEnum, Debug, PartialEq)]
pub enum LogMode {
    Quiet,
    Plain,
    Fancy,
}

pub enum Logger {
    Quiet(QuietLogger),
    Plain(PlainLogger),
    Fancy(FancyLogger),
}

impl Logger {
    pub fn new(mode: Option<LogMode>) -> Self {
        match mode.unwrap_or(LogMode::Fancy) {
            LogMode::Quiet => Logger::Quiet(QuietLogger),
            LogMode::Plain => Logger::Plain(PlainLogger),
            LogMode::Fancy => Logger::Fancy(FancyLogger::new()),
        }
    }

    pub fn add_spinner(&self, msg: Option<String>) -> MgdlResult<MaybeSpinner> {
        match self {
            Logger::Quiet(_) => Ok(MaybeSpinner::new(None, None, LogMode::Quiet)?),
            Logger::Plain(_) => Ok(MaybeSpinner::new(None, None, LogMode::Plain)?),
            Logger::Fancy(logger) => logger.add_spinner(msg),
        }
    }

    pub fn finish_spinner(&self, spinner: MaybeSpinner) {
        match self {
            Logger::Quiet(_) | Logger::Plain(_) => spinner.finish_and_clear(None),
            Logger::Fancy(logger) => logger.finish_spinner(spinner),
        }
    }

    pub fn add_bar(&self, size: u64) -> MgdlResult<MaybeBar> {
        match self {
            Logger::Quiet(_) => Ok(MaybeBar::new(None, 0, LogMode::Quiet)?),
            Logger::Plain(_) => Ok(MaybeBar::new(None, 0, LogMode::Plain)?),
            Logger::Fancy(logger) => logger.add_bar(size),
        }
    }

    pub fn finish_bar(&self, bar: MaybeBar) {
        match self {
            Logger::Quiet(_) | Logger::Plain(_) => bar.finish_and_clear(None),
            Logger::Fancy(logger) => logger.finish_bar(bar),
        }
    }
}

pub struct QuietLogger;
pub struct PlainLogger;

pub struct FancyLogger {
    multi_progress: MultiProgress,
}

impl FancyLogger {
    fn new() -> Self {
        Self {
            multi_progress: MultiProgress::new(),
        }
    }

    fn add_spinner(&self, msg: Option<String>) -> MgdlResult<MaybeSpinner> {
        Ok(MaybeSpinner::new(
            Some(&self.multi_progress),
            msg,
            LogMode::Fancy,
        )?)
    }

    fn finish_spinner(&self, spinner: MaybeSpinner) {
        spinner.finish_and_clear(Some(&self.multi_progress));
    }

    fn add_bar(&self, size: u64) -> MgdlResult<MaybeBar> {
        Ok(MaybeBar::new(
            Some(&self.multi_progress),
            size,
            LogMode::Fancy,
        )?)
    }

    fn finish_bar(&self, bar: MaybeBar) {
        bar.finish_and_clear(Some(&self.multi_progress));
    }
}
