use crate::{
    logger::LogMode,
    utils::{gen_progress_bar, gen_progress_spinner},
    MgdlResult,
};

use indicatif::{MultiProgress, ProgressBar};

pub struct MaybeSpinner {
    inner: Option<ProgressBar>,
    mode: LogMode,
}

impl MaybeSpinner {
    pub fn new(
        multi_progress: Option<&MultiProgress>,
        message: Option<String>,
        log_mode: LogMode,
    ) -> MgdlResult<Self> {
        let inner = if log_mode == LogMode::Fancy {
            let spinner = gen_progress_spinner()?;
            let spinner = match multi_progress {
                Some(mp) => mp.add(spinner),
                None => spinner,
            };

            if let Some(msg) = &message {
                spinner.set_message(msg.clone());
            }

            Some(spinner)
        } else {
            if let Some(msg) = &message {
                if log_mode == LogMode::Plain {
                    println!("[INFO] {msg}");
                }
            }
            None
        };

        Ok(Self {
            inner,
            mode: log_mode,
        })
    }

    pub fn set_message(&self, msg: String) {
        match self.mode {
            LogMode::Fancy => {
                if let Some(s) = &self.inner {
                    s.set_message(msg);
                }
            }
            LogMode::Plain => {
                println!("[INFO] {}", msg);
            }
            LogMode::Quiet => {}
        }
    }

    pub fn finish_and_clear(self, multi_progress: Option<&MultiProgress>) {
        if let Some(bar) = self.inner {
            if self.mode == LogMode::Fancy {
                if let Some(mp) = multi_progress {
                    bar.finish_and_clear();
                    mp.remove(&bar);
                }
            }
        }
    }
}

pub struct MaybeBar {
    inner: Option<ProgressBar>,
    mode: LogMode,
}

impl MaybeBar {
    pub fn new(
        multi_progress: Option<&MultiProgress>,
        size: u64,
        log_mode: LogMode,
    ) -> MgdlResult<Self> {
        let inner = if log_mode == LogMode::Fancy {
            let bar = gen_progress_bar(size)?;
            Some(match multi_progress {
                Some(mp) => mp.add(bar),
                None => bar,
            })
        } else {
            None
        };

        Ok(Self {
            inner,
            mode: log_mode,
        })
    }

    pub fn set_prefix(&self, msg: String) {
        if let Some(bar) = &self.inner {
            bar.set_prefix(msg);
        } else if self.mode == LogMode::Plain {
            println!("[INFO] {msg}");
        }
    }

    pub fn inc(&self, delta: u64) {
        if let Some(bar) = &self.inner {
            bar.inc(delta);
        }
    }

    pub fn println(&self, msg: String) {
        match self.mode {
            LogMode::Fancy => {
                if let Some(bar) = &self.inner {
                    bar.println(msg);
                }
            }
            LogMode::Plain => println!("[INFO] {msg}"),
            LogMode::Quiet => {}
        }
    }

    pub fn finish_and_clear(self, multi_progress: Option<&MultiProgress>) {
        if let Some(bar) = self.inner {
            if self.mode == LogMode::Fancy {
                if let Some(mp) = multi_progress {
                    bar.finish_and_clear();
                    mp.remove(&bar);
                }
            }
        }
    }
}
