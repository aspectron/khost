use crate::imports::*;
pub use ::console::style;
use ::console::{Emoji, Style};
pub use cliclack::confirm;
use cliclack::{Theme, ThemeState};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use workflow_core::enums::Describe;

lazy_static::lazy_static! {
    pub static ref SIGTERM: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    pub static ref INTERACTION: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

pub fn init_user_interaction() {
    ctrlc::set_handler(move || {
        SIGTERM.store(true, Ordering::Relaxed);
    })
    .expect("setting Ctrl-C handler");

    cliclack::set_theme(ActionTheme);
}

pub fn terminate() {
    SIGTERM.store(true, Ordering::Relaxed);
}

pub fn was_interaction_cancelled() -> bool {
    INTERACTION.load(Ordering::Relaxed) && !SIGTERM.load(Ordering::Relaxed)
}

pub fn cancel_color() -> Style {
    if was_interaction_cancelled() {
        Style::new().dim()
    } else {
        Style::new().red()
    }
}

pub trait Action: Describe + Clone + Copy + Eq {
    // type Error: Display + From<std::io::Error>;

    fn select(ctx: &mut Context) -> Result<()> {
        let mut selector = cliclack::select(Self::caption());
        for action in Self::iter() {
            selector = selector.item(*action, action.describe(), action.rustdoc());
        }

        loop {
            INTERACTION.store(true, Ordering::Relaxed);
            let selection = selector.interact();
            INTERACTION.store(false, Ordering::Relaxed);

            match selection {
                Ok(selection) => match selection.main(ctx) {
                    Ok(_) => {
                        break;
                    }
                    Err(e) => {
                        if SIGTERM.load(Ordering::Relaxed) {
                            return Err(e);
                        } else {
                            cliclack::log::error(e.to_string()).ok();
                        }
                    }
                },
                Err(e) => {
                    if SIGTERM.load(Ordering::Relaxed) {
                        println!("SIGTERM - ERROR");
                        return Err(e.into());
                    } else {
                        println!("SIGTERM - OK");
                        // return Err(e.into());
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }

    fn run(ctx: &mut Context) -> Result<()> {
        let mut selector = cliclack::select(Self::caption());
        for action in Self::iter() {
            selector = selector.item(*action, action.describe(), action.rustdoc());
        }

        loop {
            selector.interact()?.main(ctx)?;
        }
    }

    fn main(&self, _ctx: &mut Context) -> Result<()>;
}

const S_BAR: Emoji = Emoji("│", "|");
const S_BAR_END: Emoji = Emoji("└", "—");

pub struct ActionTheme;

impl Theme for ActionTheme {
    fn state_symbol_color(&self, state: &ThemeState) -> Style {
        match state {
            ThemeState::Submit => Style::new().green(),
            ThemeState::Cancel => cancel_color(),
            _ => self.bar_color(state),
        }
    }

    fn bar_color(&self, state: &ThemeState) -> Style {
        match state {
            ThemeState::Active => Style::new().cyan(),
            ThemeState::Cancel => cancel_color(),
            ThemeState::Submit => Style::new().bright().black(),
            ThemeState::Error(_) => Style::new().yellow(),
        }
    }

    fn format_footer_with_message(&self, state: &ThemeState, message: &str) -> String {
        format!(
            "{}\n", // '\n' vanishes by style applying, thus exclude it from styling
            self.bar_color(state).apply_to(match state {
                ThemeState::Active => format!("{S_BAR_END}  {message}"),
                ThemeState::Cancel => {
                    if was_interaction_cancelled() {
                        format!("{S_BAR}")
                    } else {
                        format!("{S_BAR_END}  Operation cancelled.")
                    }
                }
                ThemeState::Submit => format!("{S_BAR}"),
                ThemeState::Error(err) => format!("{S_BAR_END}  {err}"),
            })
        )
        // }
    }
}

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
