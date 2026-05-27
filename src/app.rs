use std::io::{Result, Write, stdout};
use std::thread;
use std::time::{Duration, Instant};

use crate::input::{InputEvent, poll_event};
use crate::life::Board;
use crate::render::render_frame;
use crate::screen::TerminalSession;

enum Mode {
    Normal,
    Screensaver,
}

pub struct App {
    mode: Mode,
    board: Board,
    width: usize,
    height: usize,
    last_activity: Instant,
    idle_timeout: Duration,
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
            mode: Mode::Normal,
            board,
            width,
            height,
            last_activity: Instant::now(),
            idle_timeout: Duration::from_secs(5), // default 5 seconds inactivity timeout
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

    fn run_screensaver(&mut self) -> Result<()> {
        let _screen = TerminalSession::enter()?;
        let mut out = stdout();

        self.mode = Mode::Screensaver;

        loop {
            match poll_event(Duration::from_millis(100))? {
                InputEvent::KeyPress(_, _) => break,

                InputEvent::Resize(w, h) => {
                    self.resize_board(w, h);
                }

                InputEvent::None => {}
            }

            //advance Conway
            self.board.step_ahead();

            //build frame
            let frame = render_frame(&self.board);

            //cursor-home redraw
            write!(out, "\x1B[H")?;
            out.write_all(frame.as_bytes())?;
            out.flush()?;

            //frame pacing
            thread::sleep(Duration::from_millis(100));
        }

        self.mode = Mode::Normal;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        println!("TIDE active. Monitoring terminal inactivity... (Timeout: 5s, Press 'q', 'Esc', or 'Ctrl+C' to exit)");

        // Enable raw mode to monitor inputs non-blockingly
        crossterm::terminal::enable_raw_mode()?;

        self.last_activity = Instant::now();

        loop {
            // Poll for input lightly to ensure near-zero CPU usage
            match poll_event(Duration::from_millis(50))? {
                InputEvent::KeyPress(code, modifiers) => {
                    // Check for exit keys: 'q', Esc, or Ctrl+C
                    if code == crossterm::event::KeyCode::Char('q')
                        || code == crossterm::event::KeyCode::Esc
                        || (code == crossterm::event::KeyCode::Char('c') && modifiers.contains(crossterm::event::KeyModifiers::CONTROL))
                    {
                        println!("\r\nExiting TIDE. Goodbye!");
                        break;
                    }
                    // Reset timer on other keys
                    self.last_activity = Instant::now();
                }

                InputEvent::Resize(w, h) => {
                    self.resize_board(w, h);
                    self.last_activity = Instant::now();
                }

                InputEvent::None => {}
            }

            if self.last_activity.elapsed() >= self.idle_timeout {
                // Temporarily disable raw mode to let run_screensaver manage its own TerminalSession
                let _ = crossterm::terminal::disable_raw_mode();

                self.run_screensaver()?;

                // Re-enable raw mode for normal monitoring
                crossterm::terminal::enable_raw_mode()?;

                // Reset timer when screensaver exits
                self.last_activity = Instant::now();
            }

            thread::sleep(Duration::from_millis(50));
        }

        let _ = crossterm::terminal::disable_raw_mode();
        Ok(())
    }
}
