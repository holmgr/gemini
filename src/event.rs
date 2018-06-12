use game::Game;
use std::{io,
          sync::{mpsc::{channel, Receiver, Sender},
                 Arc,
                 Mutex},
          thread::{park_timeout, spawn},
          time::{Duration, Instant}};
use termion::{event, input::TermRead};

use gui::dialog::Dialog;

/// User and system events.
#[derive(Clone)]
pub enum Event {
    Input(event::Key),
    Update,
    Travel,
    Refuel,
    Dock(usize),
    Undock(usize),
    AutosaveStarted,
    AutosaveCompleted,
    OpenDialog(Arc<Mutex<Dialog>>),
    CloseDialog,
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
        spawn(|| loop {
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
    spawn(move || {
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

/// Start listener for events that should update player state.
pub fn add_player_handler(state: Arc<Game>) {
    let rx = HANDLER.recv_handle();
    let sx = HANDLER.send_handle();
    spawn(move || {
        loop {
            let evt = rx.recv().unwrap();
            match evt {
                Event::Travel | Event::Dock(_) | Event::Undock(_) | Event::Refuel => {
                    match evt {
                        Event::Dock(planet_id) => {
                            if let Ok(mut player) = state.player.lock() {
                                player.dock(planet_id);
                            }
                        }
                        Event::Undock(_) => {
                            if let Ok(mut player) = state.player.lock() {
                                player.undock();
                            }
                        }
                        Event::Refuel => {
                            if let Ok(mut player) = state.player.lock() {
                                player.refuel();
                            }
                        }
                        _ => {}
                    };
                    sx.send(Event::AutosaveStarted).unwrap();
                    // Only need to save player.
                    state.save_player();
                    sx.send(Event::AutosaveCompleted).unwrap();
                }
                _ => {}
            };
        }
    });
}

/// Start listener for events that should run an update on the game state.
pub fn add_update_handler(state: Arc<Game>) {
    let sx = HANDLER.send_handle();
    let timeout_freq = Duration::from_secs(10);
    let mut beginning_park = Instant::now();
    let mut timeout_remaining = timeout_freq;
    spawn(move || {
        // Update right away first time.
        state.update();
        sx.send(Event::Update).unwrap();
        loop {
            // Wait uptil 10s, must check.
            park_timeout(timeout_remaining);
            let elapsed = beginning_park.elapsed();
            // If timeout reached, send event and reset timer.
            if elapsed >= timeout_freq {
                state.update();
                sx.send(Event::Update).unwrap();
                timeout_remaining = timeout_freq;
                beginning_park = Instant::now();
            } else {
                timeout_remaining = timeout_freq - elapsed;
            }
        }
    });
}
