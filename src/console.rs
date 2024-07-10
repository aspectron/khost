use crate::imports::*;
pub use cliclack::confirm;
use std::cell::RefCell;

pub struct Progress {
    inner: Option<cliclack::ProgressBar>,
    status: RefCell<bool>,
}

impl Progress {
    pub fn new(inner: Option<cliclack::ProgressBar>) -> Self {
        Self {
            inner,
            status: RefCell::new(false),
        }
    }

    pub fn inc(&self, n: usize) {
        if let Some(progress) = &self.inner {
            progress.inc(n as u64);
        }
    }

    pub fn set_message<S: Display>(&self, message: S) {
        if let Some(progress) = &self.inner {
            progress.set_message(message);
        }
    }

    pub fn start<S: Display>(&self, message: S) -> Result<()> {
        if let Some(progress) = &self.inner {
            progress.start(message)
        } else {
            log::step(&message)?;
        }
        Ok(())
    }

    pub fn stop<S: Display>(&self, message: S) -> Result<()> {
        *self.status.borrow_mut() = true;
        if let Some(progress) = &self.inner {
            progress.stop(message)
        }
        Ok(())
    }

    pub fn done<S: Display>(&self, message: S) -> Result<()> {
        *self.status.borrow_mut() = true;
        if let Some(progress) = &self.inner {
            progress.stop(message)
        }
        Ok(())
    }

    pub fn cancel<S: Display>(&self, message: S) -> Result<()> {
        *self.status.borrow_mut() = true;
        if let Some(progress) = &self.inner {
            progress.cancel(message)
        }
        Ok(())
    }

    pub fn error<S: Display>(&self, message: S) -> Result<()> {
        *self.status.borrow_mut() = true;
        if let Some(progress) = &self.inner {
            progress.error(message)
        }
        Ok(())
    }
}

pub fn step<S, F>(caption: S, f: F) -> Result<()>
where
    S: Display,
    F: FnOnce() -> Result<()>,
{
    let progress = not_verbose().then(cliclack::spinner);

    if let Some(progress) = progress.as_ref() {
        progress.start(&caption);
    } else {
        let _ = log::step(&caption);
    }

    match f() {
        Ok(_) => {
            if let Some(progress) = progress.as_ref() {
                progress.stop(caption);
            }

            Ok(())
        }
        Err(e) => {
            if let Some(progress) = progress.as_ref() {
                progress.error(e.to_string());
            }
            Err(e)
        }
    }
}

pub fn track<S, F>(caption: S, f: F) -> Result<()>
where
    S: Display,
    F: FnOnce(&Progress) -> Result<()>,
{
    let progress = Progress::new(not_verbose().then(cliclack::spinner));
    progress.start(&caption)?;

    match f(&progress) {
        Ok(_) => {
            if !*progress.status.borrow() {
                if let Some(progress) = progress.inner.as_ref() {
                    progress.stop(caption);
                }
            }

            Ok(())
        }
        Err(e) => {
            let _ = progress.error(e.to_string());
            Err(e)
        }
    }
}

pub fn progress<S, R, F>(n: usize, caption: S, f: F) -> Result<()>
where
    S: Display,
    R: Display,
    F: FnOnce(&Progress) -> Result<R>,
{
    let progress = not_verbose().then(|| cliclack::progress_bar(n as u64));

    if let Some(progress) = progress.as_ref() {
        progress.start(&caption);
    } else {
        let _ = log::step(&caption);
    }

    let progress = Progress::new(progress);

    match f(&progress) {
        Ok(msg) => {
            if let Some(progress) = progress.inner.as_ref() {
                progress.stop(msg);
            }

            Ok(())
        }
        Err(e) => {
            if let Some(progress) = progress.inner.as_ref() {
                progress.error(e.to_string());
            }
            Err(e)
        }
    }
}
