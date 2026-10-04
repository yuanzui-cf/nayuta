use crate::infra::error::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::stdout,
    sync::atomic::{AtomicBool, Ordering},
};

pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);
pub struct Guard;
pub fn restore() {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(
        stdout(),
        DisableMouseCapture,
        DisableBracketedPaste,
        Show,
        LeaveAlternateScreen
    );
}
impl Guard {
    pub fn enter(mouse: bool) -> Result<Self> {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            hook(info);
        }));
        ctrlc::set_handler(|| {
            INTERRUPTED.store(true, Ordering::SeqCst);
        })
        .map_err(|e| crate::infra::error::user("error", e.to_string()))?;
        let guard = Self;
        terminal::enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, EnableBracketedPaste, Hide)?;
        if mouse {
            execute!(stdout(), EnableMouseCapture)?;
        }
        Ok(guard)
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        restore();
    }
}
