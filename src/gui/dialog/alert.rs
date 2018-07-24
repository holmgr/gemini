use super::*;

type Action = Fn(&mut Sender<Event>) -> Option<GUIEvent>;

/// Alert dialog window.
pub struct AlertDialog(MultiDialog);

impl AlertDialog {
    /// Create a new alert dialog.
    pub fn new(title: String, action: Box<Action>) -> Self {
        AlertDialog {
            0: MultiDialog::new(title, vec![("OK", action)]),
        }
    }
}

impl Dialog for AlertDialog {
    /// Returns the title string describing the dialog box.
    fn title(&self) -> String {
        self.0.title()
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) -> Option<GUIEvent> {
        self.0.handle_event(event)
    }

    /// Draws the dialog in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        self.0.draw(term, area);
    }
}
