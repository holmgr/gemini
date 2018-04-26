use super::*;

/// Displays the market tab.
pub struct MarketTab {
    state: Arc<Game>,
    sender: Sender<Event>,
}

impl Tab for MarketTab {
    /// Creates a market tab.
    fn new(state: Arc<Game>, send_handle: Sender<Event>) -> Box<Self> {
        Box::new(MarketTab {
            state,
            sender: send_handle,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Market")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, _event: Event) {}

    /// Draws the tab in the given terminal and area.
    fn draw(&self, _term: &mut Terminal<MouseBackend>, _area: &Rect) {}
}
