use crate::{
    utils::{gen_progress_bar, gen_progress_spinner},
    MgdlResult,
};
use indicatif::{MultiProgress, ProgressBar};

pub struct MaybeSpinner(Option<ProgressBar>);

impl MaybeSpinner {
    pub fn new(multi_progress: Option<&MultiProgress>, message: Option<String>) -> MgdlResult<Self> {
        let spinner = match multi_progress {
            Some(multi_progress) => {
                let spinner = multi_progress.add(gen_progress_spinner()?);
                if let Some(message) = message {
                    spinner.set_message(message);
                }
                Some(spinner)
            }
            None => None,
        };
        Ok(Self(spinner))
    }

    pub fn set_message(&self, msg: String) {
        if let Some(s) = &self.0 {
            s.set_message(msg);
        }
    }

    pub fn finish_and_clear(self, multi_progress: Option<&MultiProgress>) {
        let Some(spinner) = self.0 else { return };
        let Some(multi_progress) = multi_progress else {
            return;
        };

        spinner.finish_and_clear();
        multi_progress.remove(&spinner);
    }
}

pub struct MaybeBar(Option<ProgressBar>);

impl MaybeBar {
    pub fn new(multi_progress: Option<&MultiProgress>, size: u64) -> MgdlResult<Self> {
        let bar = match multi_progress {
            Some(multi_progress) => {
                let bar = multi_progress.add(gen_progress_bar(size)?);
                Some(bar)
            }
            None => None,
        };
        Ok(Self(bar))
    }

    pub fn set_prefix(&self, msg: String) {
        if let Some(bar) = &self.0 {
            bar.set_prefix(msg)
        }
    }

    pub fn inc(&self, delta: u64) {
        if let Some(bar) = &self.0 {
            bar.inc(delta);
        }
    }

    pub fn println(&self, msg: String) {
        if let Some(bar) = &self.0 {
            bar.println(msg);
        }
    }

    pub fn finish_and_clear(self, progress: Option<&MultiProgress>) {
        let Some(bar) = self.0 else { return };
        let Some(progress) = progress else { return };

        bar.finish_and_clear();
        progress.remove(&bar);
    }
}
