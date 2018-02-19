use super::*;

/// Displays the shipyard tab.
pub struct ShipyardTab {
    state: Arc<Game>,
}

impl Tab for ShipyardTab {
    /// Creates a shipyard tab.
    fn new(state: Arc<Game>) -> Box<Self> {
        Box::new(ShipyardTab { state: state })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Shipyard")
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {}
}
