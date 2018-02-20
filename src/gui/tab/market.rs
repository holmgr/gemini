use super::*;

/// Displays the market tab.
pub struct MarketTab {
    state: Arc<Game>,
}

impl Tab for MarketTab {
    /// Creates a market tab.
    fn new(state: Arc<Game>) -> Box<Self> {
        Box::new(MarketTab { state: state })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Market")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) {}

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {}
}
