use std::sync::{mpsc::Sender, Arc};
use tui::{backend::MouseBackend, layout::Rect, Terminal};

use super::GUIEvent;
use event::{Event, HANDLER};
use game::Game;

mod galaxymap;
mod market;
mod mission;
mod shipyard;
mod status;
mod systemmap;

/// Interface for dealing with visual tabs in the GUI.
pub trait Tab {
    /// Creates a new tab.
    fn new(_: Arc<Game>, _: Sender<Event>) -> Box<Self>
    where
        Self: Sized;

    /// Returns the title string describing the tab.
    fn title(&self) -> String;

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) -> Option<GUIEvent>;

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect);
}

/// Returns a vector of tabs to be used.
pub fn create_tabs(state: &Arc<Game>) -> Vec<Box<dyn Tab>> {
    vec![
        status::StatusTab::new(state.clone(), HANDLER.send_handle()),
        galaxymap::GalaxyMapTab::new(state.clone(), HANDLER.send_handle()),
        systemmap::SystemMapTab::new(state.clone(), HANDLER.send_handle()),
        market::MarketTab::new(state.clone(), HANDLER.send_handle()),
        mission::MissionTab::new(state.clone(), HANDLER.send_handle()),
        shipyard::ShipyardTab::new(state.clone(), HANDLER.send_handle()),
    ]
}
