use super::*;

/// Displays the mission tab.
pub struct MissionTab {
    state: Arc<Game>,
}

impl Tab for MissionTab {
    /// Creates a mission tab.
    fn new(state: Arc<Game>) -> Box<Self> {
        Box::new(MissionTab { state: state })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Missions")
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {}
}
