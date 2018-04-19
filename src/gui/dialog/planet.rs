use super::*;
use tui::{layout::{Direction, Group, Rect, Size}, style::{Color, Style},
          widgets::{Block, Borders, Paragraph, SelectableList, Widget}};

use player::PlayerState;

/// Dialog window representing options available when interacting with a planet.
#[derive(Clone)]
pub struct PlanetDialog {
    sender: Sender<Event>,
    planetid: usize,
    title: String,
    selected: usize,
    buttons: Vec<&'static str>,
}

impl PlanetDialog {
    /// Create a new PlanetDialog.
    pub fn new(planetid: usize, is_landed: bool, title: String) -> Box<Self> {
        let buttons = match is_landed {
            true => vec!["Refuel", "Undock"],
            false => vec!["Dock"],
        };

        Box::new(PlanetDialog {
            sender: HANDLER.send_handle(),
            planetid,
            title,
            selected: 0,
            buttons,
        })
    }
}

impl Dialog for PlanetDialog {
    /// Returns the title string describing the dialog box.
    fn title(&self) -> String {
        self.title.clone()
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Input(input) => {
                self.selected = match input {
                    // Move up.
                    keyevent::Key::Char('k') => self.selected.max(1) - 1,
                    // Move down.
                    keyevent::Key::Char('j') => (self.selected + 1).min(self.buttons.len() - 1),
                    _ => self.selected,
                };
                match input {
                    keyevent::Key::Char('\n') => {
                        // Send the correct event.
                        match self.buttons[self.selected] {
                            "Refuel" => {
                                self.sender.send(Event::Refuel);
                            }
                            "Undock" => {
                                self.sender.send(Event::Undock(self.planetid));
                            }
                            "Dock" => {
                                self.sender.send(Event::Dock(self.planetid));
                            }
                            _ => {}
                        };
                        self.sender.send(Event::CloseDialog);
                    }
                    keyevent::Key::Backspace => {
                        self.sender.send(Event::CloseDialog);
                    }
                    _ => {}
                };
            }
            _ => {}
        };
    }

    /// Draws the dialog in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        let dialog_rect = Rect::new((area.width - 60) / 2, (area.height - 5) / 2, 60, 5);
        let labels: Vec<String> = self.buttons
            .iter()
            .map(|s| format!("{:^1$}", s, 60))
            .collect::<Vec<_>>();
        Group::default()
            .direction(Direction::Vertical)
            .sizes(&[Size::Fixed(self.buttons.len() as u16)])
            .render(term, &dialog_rect, |term, chunks| {
                SelectableList::default()
                    .items(&labels)
                    .select(self.selected)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(&format!("Planet: {}", &self.title)),
                    )
                    .style(Style::default().fg(Color::Green).bg(Color::DarkGray))
                    .highlight_style(Style::default().fg(Color::Yellow).bg(Color::Gray))
                    .render(term, &chunks[0]);
            });
    }

    /// Returns a deep clone.
    fn box_clone(&self) -> Box<Dialog> {
        Box::new((*self).clone())
    }
}
