use std::sync::Arc;

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::layout::Rect;

use game::Game;

mod status;
mod map;
mod market;
mod mission;
mod shipyard;

/// Interface for dealing with visual tabs in the GUI.
pub trait Tab {
    /// Creates a new tab.
    fn new(Arc<Game>) -> Box<Self>
    where
        Self: Sized;

    /// Returns the title string describing the tab.
    fn title(&self) -> String;

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect);
}

/// Returns a vector of tabs to be used.
pub fn create_tabs(state: Arc<Game>) -> Vec<Box<Tab>> {
    vec![
        status::StatusTab::new(state.clone()),
        map::MapTab::new(state.clone()),
        market::MarketTab::new(state.clone()),
        mission::MissionTab::new(state.clone()),
        shipyard::ShipyardTab::new(state.clone()),
    ]
}
