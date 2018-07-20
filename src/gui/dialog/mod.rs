use std::sync::mpsc::Sender;
use termion::event as keyevent;
use tui::{backend::MouseBackend, layout::Rect, Terminal};

use super::GUIEvent;
use event::{Event, HANDLER};

mod multi;

pub use self::multi::MultiDialog;

/// A dialog box.
pub trait Dialog: Send {
    /// Returns the title string describing the dialog box.
    fn title(&self) -> String;

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) -> Option<GUIEvent>;

    /// Draws the dialog in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect);
}
