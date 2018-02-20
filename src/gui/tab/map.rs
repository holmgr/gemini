use super::*;

/// Displays the map tab.
pub struct MapTab {
    state: Arc<Game>,
}

impl Tab for MapTab {
    /// Creates a map tab.
    fn new(state: Arc<Game>) -> Box<Self> {
        Box::new(MapTab { state: state })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Galaxy Map")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) {}

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {}
}
