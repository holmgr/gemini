use super::*;

/// Displays the status tab.
pub struct StatusTab {
    state: Arc<Game>,
}

impl Tab for StatusTab {
    /// Creates a status tab.
    fn new(state: Arc<Game>) -> Box<Self> {
        Box::new(StatusTab { state: state })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Status")
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {}
}
