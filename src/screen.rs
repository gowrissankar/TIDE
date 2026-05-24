//terminal shir

//RAII style : everythign tied to session , easy cleanup

//importing req commands
use std::io::{Result, stdout};

use crossterm::{
    cursor::{Hide, Show},
    execute, //for terminal commands
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

pub struct TerminalSession;

impl TerminalSession {
    pub fn enter() -> Result<Self> {
        //raw mod eon
        enable_raw_mode()?; //raw terminal , ? : error catcher exit early if error

        let mut out = stdout();

        //send teminla commands to stdout
        if let Err(err) = execute!(out, EnterAlternateScreen, Hide, Clear(ClearType::All)) {
            //rollback raw mode if terminal enter fails
            let _ = disable_raw_mode();
            return Err(err);
        }
        Ok(Self)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let mut out = stdout();

        //best effort cleanup
        let _ = execute!(out, Show, LeaveAlternateScreen,);

        let _ = disable_raw_mode(); //  _ throwaway binding : loose result
    }
}
