mod app;
mod input;
mod life;
mod render;
mod screen;

use life::Board;
use render::render_frame;

use screen::TerminalSession;
use std::{thread, time::Duration};

fn main() -> std::io::Result<()> {
    let _screen = TerminalSession::enter()?;

    thread::sleep(Duration::from_secs(3));

    Ok(())
}
