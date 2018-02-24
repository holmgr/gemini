use std::sync::mpsc::{channel, Receiver, Sender};
use std::io;
use std::thread;
use std::sync::Mutex;

use termion::event;
use termion::input::TermRead;

/// User and system events.
#[derive(Clone)]
pub enum Event {
    Input(event::Key),
    Update,
}

lazy_static! {
    pub static ref HANDLER: EventHandler = EventHandler::new();
}

/// Accepts events from all senders and propagates them to all recievers.
pub struct EventHandler {
    sender: Mutex<Sender<Event>>,
    receiver: Mutex<Receiver<Event>>,
    listeners: Mutex<Vec<Sender<Event>>>,
}

impl EventHandler {
    /// Create a new EventHandler.
    fn new() -> EventHandler {
        let (tx, rx) = channel();
        EventHandler {
            sender: Mutex::new(tx),
            receiver: Mutex::new(rx),
            listeners: Mutex::new(vec![]),
        }
    }

    /// Start the global event handler.
    pub fn start() {
        thread::spawn(|| loop {
            let evt = HANDLER.receiver.lock().unwrap().recv().unwrap();
            for listener in HANDLER.listeners.lock().unwrap().iter() {
                listener.send(evt.clone()).unwrap();
            }
        });
    }

    /// Get a sender handle which can be used to send events to all recievers.
    pub fn send_handle(&self) -> Sender<Event> {
        self.sender.lock().unwrap().clone()
    }

    /// Get a receiver handle which will be called when events are dispatched.
    pub fn recv_handle(&self) -> Receiver<Event> {
        let (tx, rx) = channel();
        self.listeners.lock().unwrap().push(tx);
        rx
    }
}

/// Start listener for keyboard events and forward to event handler.
pub fn add_keyboard_handler() {
    let send_handle = HANDLER.send_handle();
    thread::spawn(move || {
        let stdin = io::stdin();
        for c in stdin.keys() {
            let evt = c.unwrap();
            send_handle.send(Event::Input(evt)).unwrap();
            if evt == event::Key::Char('q') {
                break;
            }
        }
    });
}
