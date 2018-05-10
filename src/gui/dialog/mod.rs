use std::sync::{Arc, mpsc::Sender};
use termion::event as keyevent;
use tui::{Terminal, backend::MouseBackend, layout::Rect};

use event::{Event, HANDLER};

mod multi;

pub use self::multi::MultiDialog;

/// A dialog box.
pub trait Dialog: Send {
    /// Returns the title string describing the dialog box.
    fn title(&self) -> String;

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event);

    /// Draws the dialog in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect);
}
