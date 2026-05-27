//terminal shir

//RAII style : everythign tied to session , easy cleanup

//importing req commands
use std::io::{Result, stdout};

use crossterm::{
    cursor::{Hide, Show},
    execute, //for terminal commands
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode, EnableLineWrap, DisableLineWrap
    },
};

pub struct TerminalSession;

impl TerminalSession {
    pub fn enter() -> Result<Self> {
        // Setup robust panic hook for terminal cleanup
        std::panic::set_hook(Box::new(|info| {
            Self::cleanup();
            eprintln!("\nPanic occurred: {}", info);
        }));

        enable_raw_mode()?;

        let mut out = stdout();
        if let Err(err) = execute!(out, EnterAlternateScreen, Hide, Clear(ClearType::All), DisableLineWrap) {
            let _ = disable_raw_mode();
            return Err(err);
        }
        Ok(Self)
    }

    pub fn cleanup() {
        let mut out = stdout();
        // best effort cleanup, explicitly restoring cursor, alternate screen, and line wrapping
        let _ = execute!(
            out,
            Show,
            LeaveAlternateScreen,
            EnableLineWrap
        );
        let _ = disable_raw_mode();
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        Self::cleanup();
    }
}
