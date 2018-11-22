use super::*;

/// Empty View used as baseline.
pub struct Empty {}

impl Empty {
    /// Create a new empty view.
    pub fn new() -> Self {
        Empty {}
    }
}

impl View for Empty {
    /// Handles the given input in the view.
    fn handle_event(&mut self, event: Event) -> Trans {
        Trans::None
    }
}
