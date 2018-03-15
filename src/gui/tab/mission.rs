use super::*;

/// Displays the mission tab.
pub struct MissionTab {
    state: Arc<Game>,
    sender: Sender<Event>,
}

impl Tab for MissionTab {
    /// Creates a mission tab.
    fn new(state: Arc<Game>, send_handle: Sender<Event>) -> Box<Self> {
        Box::new(MissionTab {
            state: state,
            sender: send_handle,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Missions")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) {}

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {}
}
