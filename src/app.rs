use std::io::{Result, Write, stdout};
use std::time::Duration;

use crate::input::{InputEvent, poll_event};
use crate::life::Board;
use crate::render::render_frame;
use crate::screen::TerminalSession;

pub struct App {
    board: Board,
    width: usize,
    height: usize,
}

impl App {
    pub fn new() -> Self {
        // query terminal size
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        
        // subtract 1 from columns to prevent auto-wrap double spacing
        let width = if cols > 1 { cols as usize - 1 } else { 1 };
        let height = (rows as usize) * 2;

        let board = Board::new(width, height);

        let mut app = Self {
            board,
            width,
            height,
        };
        app.seed_glider();
        app
    }

    fn seed_glider(&mut self) {
        let cx = self.width / 2;
        let cy = self.height / 2;

        // seed glider near center to avoid toroidal split at startup
        self.board.set((cx + 1) % self.width, (cy + 0) % self.height, true);
        self.board.set((cx + 2) % self.width, (cy + 1) % self.height, true);
        self.board.set((cx + 0) % self.width, (cy + 2) % self.height, true);
        self.board.set((cx + 1) % self.width, (cy + 2) % self.height, true);
        self.board.set((cx + 2) % self.width, (cy + 2) % self.height, true);
    }

    fn resize_board(&mut self, width: u16, height: u16) {
        // subtract 1 from columns to prevent auto-wrap double spacing
        self.width = if width > 1 { width as usize - 1 } else { 1 };
        self.height = (height as usize) * 2;

        self.board = Board::new(self.width, self.height);
        self.seed_glider();
    }

    pub fn run(&mut self) -> Result<()> {
        let _screen = TerminalSession::enter()?;
        let mut out = stdout();

        loop {
            // Frame pacing is driven purely by poll_event timeout
            match poll_event(Duration::from_millis(100))? {
                InputEvent::KeyPress(_, _) => break,

                InputEvent::Resize(w, h) => {
                    self.resize_board(w, h);
                }

                InputEvent::None => {
                    // Timeout occurred, advance the simulation
                    self.board.step_ahead();
                }
            }

            // Build frame
            let frame = render_frame(&self.board);

            // Cursor-home redraw
            write!(out, "\x1B[H")?;
            out.write_all(frame.as_bytes())?;
            out.flush()?;
        }

        Ok(())
    }
}
