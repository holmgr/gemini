use super::*;

/// Displays the mission tab.
pub struct MissionTab {}

impl Tab for MissionTab {
    /// Creates a mission tab.
    fn new(_: Arc<Game>, _: Sender<Event>) -> Box<Self> {
        Box::new(MissionTab {})
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Missions")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, _event: Event) -> Option<GUIEvent> {
        None
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, _term: &mut Terminal<MouseBackend>, _area: &Rect) {}
}
