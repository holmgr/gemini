use super::*;
use std::sync::Mutex;
use tui::{layout::{Direction, Group, Rect, Size},
          style::{Color, Style},
          widgets::{Block, Borders, SelectableList, Widget}};

type Action = Fn() -> Event + Send + Sync;

/// Multiple choice dialog window.
pub struct MultiDialog {
    sender: Sender<Event>,
    title: String,
    selected: usize,
    actions: Vec<(&'static str, Box<Action>)>,
}

impl MultiDialog {
    /// Create a new PlanetDialog.
    pub fn new(title: String, actions: Vec<(&'static str, Box<Action>)>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(MultiDialog {
            sender: HANDLER.send_handle(),
            title,
            selected: 0,
            actions,
        }))
    }
}

impl Dialog for MultiDialog {
    /// Returns the title string describing the dialog box.
    fn title(&self) -> String {
        self.title.clone()
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) {
        if let Event::Input(input) = event {
            self.selected = match input {
                // Move up.
                keyevent::Key::Char('k') => self.selected.max(1) - 1,
                // Move down.
                keyevent::Key::Char('j') => (self.selected + 1).min(self.actions.len() - 1),
                _ => self.selected,
            };
            match input {
                keyevent::Key::Char('\n') => {
                    // Call the appropriate action.
                    let (_, ref action_fn) = self.actions[self.selected];
                    self.sender.send(action_fn()).unwrap();
                    self.sender.send(Event::CloseDialog).unwrap();
                }
                keyevent::Key::Backspace => {
                    self.sender.send(Event::CloseDialog).unwrap();
                }
                _ => {}
            };
        }
    }

    /// Draws the dialog in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        let dialog_rect = Rect::new((area.width - 60) / 2, (area.height - 5) / 2, 60, 5);
        let labels: Vec<String> = self.actions
            .iter()
            .map(|&(s, _)| format!("{:^1$}", s, 60))
            .collect::<Vec<_>>();
        Group::default()
            .direction(Direction::Vertical)
            .sizes(&[Size::Fixed(self.actions.len() as u16)])
            .render(term, &dialog_rect, |term, chunks| {
                SelectableList::default()
                    .items(&labels)
                    .select(self.selected)
                    .block(Block::default().borders(Borders::ALL).title(&self.title))
                    .style(Style::default().fg(Color::Green).bg(Color::DarkGray))
                    .highlight_style(Style::default().fg(Color::Yellow).bg(Color::Gray))
                    .render(term, &chunks[0]);
            });
    }
}
