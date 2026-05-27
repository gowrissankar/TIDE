//app.rs , state machine + runtime

//owns:
// board state
// seed
// render loop
// resize
// keypress exit

//does NOT own:
// idle detection → tide-watch does that now
// shell lifecycle → shell hooks do that

use std::io::{Result, Write, stdout};
use std::thread;
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
        //query actual terminal size at launch
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));

        //cols - 1 : prevents auto-wrap which double-spaces every row in raw mode
        let width = if cols > 1 { cols as usize - 1 } else { 1 };

        //each terminal row = 2 conway rows , half block packing
        let height = (rows as usize) * 2;

        let board = Board::new(width, height);

        let mut app = Self { board, width, height };
        app.seed_glider();
        app
    }

    fn seed_glider(&mut self) {
        let cx = self.width / 2;
        let cy = self.height / 2;

        //seed center to avoid toroidal edge wrap weirdness on first frame
        self.board.set((cx + 1) % self.width, (cy + 0) % self.height, true);
        self.board.set((cx + 2) % self.width, (cy + 1) % self.height, true);
        self.board.set((cx + 0) % self.width, (cy + 2) % self.height, true);
        self.board.set((cx + 1) % self.width, (cy + 2) % self.height, true);
        self.board.set((cx + 2) % self.width, (cy + 2) % self.height, true);
    }

    fn resize_board(&mut self, width: u16, height: u16) {
        //rebuild at new size , reseed fresh
        self.width = if width > 1 { width as usize - 1 } else { 1 };
        self.height = (height as usize) * 2;

        self.board = Board::new(self.width, self.height);
        self.seed_glider();
    }

    pub fn run(&mut self) -> Result<()> {
        //RAII guard : enters alt-screen , hides cursor , raw mode on
        //drop restores everything even on panic
        let _screen = TerminalSession::enter()?;
        let mut out = stdout();

        loop {
            match poll_event(Duration::from_millis(100))? {
                InputEvent::KeyPress(_, _) => break, //any key kills saver

                InputEvent::Resize(w, h) => {
                    self.resize_board(w, h);
                }

                InputEvent::None => {}
            }

            //next conway gen
            self.board.step_ahead();

            //build full frame string , one write one flush , no per-cell writes
            let frame = render_frame(&self.board);
            write!(out, "\x1B[H")?;          //cursor home , no clear = no flicker
            out.write_all(frame.as_bytes())?;
            out.flush()?;

            //~10 fps
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
        //_screen drops here → alt off , cursor back , raw off
    }
}


// =============================================================================
// LEGACY : old self-contained idle detection (V0 approach)
// replaced by tide-watch + shell hooks in V1
// kept for reference , do not uncomment without removing new run()
// =============================================================================

// //mode switch
// enum Mode {
//     Normal,
//     Screensaver,
// }

// //old app had idle timer baked in
// pub struct App {
//     mode: Mode,
//     board: Board,
//     width: usize,
//     height: usize,
//     last_activity: Instant,     //tracks last keypress time
//     idle_timeout: Duration,     //how long before saver kicks in
// }

// impl App {
//
//     //old run_screensaver was separate from run() , now they are merged
//     fn run_screensaver(&mut self) -> Result<()> {
//         let _screen = TerminalSession::enter()?;
//         let mut out = stdout();
//
//         self.mode = Mode::Screensaver;
//
//         loop {
//             match poll_event(Duration::from_millis(100))? {
//                 InputEvent::KeyPress => break,     //exit saver
//
//                 InputEvent::Resize(_, _) => {
//                     //self.resize_board(w, h); //was commented out here too
//                 }
//
//                 InputEvent::None => {}
//             }
//
//             self.board.step_ahead();
//
//             let frame = render_frame(&self.board);
//             write!(out, "\x1B[H")?;
//             out.write_all(frame.as_bytes())?;
//             out.flush()?;
//
//             thread::sleep(Duration::from_millis(100));
//         }
//
//         self.mode = Mode::Normal;
//         Ok(())
//     }
//
//     //old run() : normal mode loop that watched inactivity itself
//     //problem : owned raw mode + fought shell for terminal control
//     //replaced by : tide-watch (external sleep/spawn) + shell preexec/precmd hooks
//     pub fn run(&mut self) -> Result<()> {
//         println!("TIDE active. Monitoring terminal inactivity... (Timeout: 5s, Press 'q', 'Esc', or 'Ctrl+C' to exit)");
//
//         crossterm::terminal::enable_raw_mode()?;   //raw on for polling
//
//         self.last_activity = Instant::now();
//
//         loop {
//             //poll lightly , near-zero cpu
//             match poll_event(Duration::from_millis(50))? {
//                 InputEvent::KeyPress(code, modifiers) => {
//                     //q , esc , ctrl+c → quit
//                     if code == crossterm::event::KeyCode::Char('q')
//                         || code == crossterm::event::KeyCode::Esc
//                         || (code == crossterm::event::KeyCode::Char('c') && modifiers.contains(crossterm::event::KeyModifiers::CONTROL))
//                     {
//                         println!("\r\nExiting TIDE. Goodbye!");
//                         break;
//                     }
//                     self.last_activity = Instant::now();   //reset timer on any other key
//                 }
//
//                 InputEvent::Resize(w, h) => {
//                     self.resize_board(w, h);
//                     self.last_activity = Instant::now();
//                 }
//
//                 InputEvent::None => {}
//             }
//
//             if self.last_activity.elapsed() >= self.idle_timeout {
//                 //timeout hit → drop to screensaver
//                 //disable raw so TerminalSession can take clean ownership
//                 let _ = crossterm::terminal::disable_raw_mode();
//
//                 self.run_screensaver()?;
//
//                 //back to normal , re-enable raw for monitoring
//                 crossterm::terminal::enable_raw_mode()?;
//
//                 self.last_activity = Instant::now();  //reset after saver exits
//             }
//
//             thread::sleep(Duration::from_millis(50));
//         }
//
//         let _ = crossterm::terminal::disable_raw_mode();
//         Ok(())
//     }
// }
