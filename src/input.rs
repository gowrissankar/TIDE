//terminal events

use std::io::Result;
use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, poll, read};
pub enum InputEvent {
    //rust enums stronger can carry data
    KeyPress(KeyCode, KeyModifiers), //for exit
    Resize(u16, u16), //resized in between
    None,             //polling timeout
}

pub fn poll_event(timeout: Duration) -> Result<InputEvent> {
    if !crossterm::event::poll(timeout)? {
        return Ok(InputEvent::None);
    }

    let event = crossterm::event::read()?;

    match event {
        crossterm::event::Event::Key(key) => {
            //ign repeats and releases
            if key.kind == crossterm::event::KeyEventKind::Press {
                Ok(InputEvent::KeyPress(key.code, key.modifiers))
            } else {
                Ok(InputEvent::None)
            }
        }
        crossterm::event::Event::Resize(width, height) => Ok(InputEvent::Resize(width, height)),
        _ => Ok(InputEvent::None),
    }
}
