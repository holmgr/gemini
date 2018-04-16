use std::sync::{Arc, mpsc::Sender};
use tui::{Terminal, backend::MouseBackend, layout::Rect};
use termion::event as keyevent;

use event::{Event, HANDLER};

/// A dialog box.
pub trait Dialog: Send {
    /// Returns the title string describing the dialog box.
    fn title(&self) -> String;

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event);

    /// Draws the dialog in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect);

    /// Returns a deep clone as box.
    fn box_clone(&self) -> Box<Dialog>;
}

impl Clone for Box<Dialog> {
    fn clone(&self) -> Box<Dialog> {
        self.box_clone()
    }
}
