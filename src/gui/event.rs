use std::sync::mpsc::{channel, Receiver};
use std::io;
use std::thread;

use termion::event;
use termion::input::TermRead;

/// User and system events which affects gui.
pub enum Event {
    Input(event::Key),
    Update,
}

/// Handles keyboard events from user.
pub struct EventHandler {}

impl EventHandler {
    /// Start listener for user keyboard events in new thread, terminates on 'q'.
    pub fn start() -> Receiver<Event> {
        let (tx, rx) = channel();

        thread::spawn(move || {
            let stdin = io::stdin();
            for c in stdin.keys() {
                let evt = c.unwrap();
                tx.send(Event::Input(evt)).unwrap();
                if evt == event::Key::Char('q') {
                    break;
                }
            }
        });

        rx
    }
}
