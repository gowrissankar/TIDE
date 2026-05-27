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
    if !poll(timeout)? {
        return Ok(InputEvent::None);
    }

    let event = read()?;

    match event {
        Event::Key(key) => {
            //ign repeats and releases

            if key.kind == KeyEventKind::Press {
                Ok(InputEvent::KeyPress(key.code, key.modifiers))
            } else {
                Ok(InputEvent::None)
            }
        }

        Event::Resize(width, height) => Ok(InputEvent::Resize(width, height)),

        //anything else
        _ => Ok(InputEvent::None),
    }
}
