use std::sync::Arc;
use std::sync::mpsc::Sender;

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::layout::Rect;

use game::Game;
use event::{Event, HANDLER};

mod status;
mod map;
mod market;
mod mission;
mod shipyard;

/// Interface for dealing with visual tabs in the GUI.
pub trait Tab {
    /// Creates a new tab.
    fn new(Arc<Game>, Sender<Event>) -> Box<Self>
    where
        Self: Sized;

    /// Returns the title string describing the tab.
    fn title(&self) -> String;

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event);

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect);
}

/// Returns a vector of tabs to be used.
pub fn create_tabs(state: Arc<Game>) -> Vec<Box<Tab>> {
    vec![
        status::StatusTab::new(state.clone(), HANDLER.send_handle()),
        map::MapTab::new(state.clone(), HANDLER.send_handle()),
        market::MarketTab::new(state.clone(), HANDLER.send_handle()),
        mission::MissionTab::new(state.clone(), HANDLER.send_handle()),
        shipyard::ShipyardTab::new(state.clone(), HANDLER.send_handle()),
    ]
}
