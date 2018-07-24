use super::*;

type Action = Fn(&mut Sender<Event>) -> Option<GUIEvent>;

/// Confirmation dialog window.
pub struct ConfirmDialog(MultiDialog);

impl ConfirmDialog {
    /// Create a new confirmation dialog.
    pub fn new(title: String, confirm_action: Box<Action>, cancel_action: Box<Action>) -> Self {
        ConfirmDialog {
            0: MultiDialog::new(
                title,
                vec![("Confirm", confirm_action), ("Cancel", cancel_action)],
            ),
        }
    }
}

impl Dialog for ConfirmDialog {
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
