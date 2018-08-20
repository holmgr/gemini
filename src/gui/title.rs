use event::Event;
use std::fmt;
use std::sync::mpsc::Sender;
use termion::event as keyevent;
use tui::{backend::MouseBackend, layout::Rect, Terminal};
use tui::{
    layout::{Direction, Group, Size},
    style::{Alignment, Color, Style},
    widgets::{Block, Paragraph, SelectableList, Widget},
};

use super::GUIEvent;
use gui::dialog::ConfirmDialog;

/// Actions available on the title page.
#[derive(Clone, Copy)]
enum Action {
    NewGame,
    LoadGame,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Action::NewGame => "NEW GAME",
                Action::LoadGame => "LOAD GAME",
            }
        )
    }
}

/// Title page GUI component displaying initial player options etc.
pub struct TitlePage {
    title: String,
    selected: usize,
    actions: Vec<Action>,
}

impl TitlePage {
    /// Creates a new title page.
    pub fn new() -> Self {
        let title = String::from(include_str!("../../res/title.txt"));
        let actions = vec![Action::NewGame, Action::LoadGame];
        TitlePage {
            title,
            selected: 0,
            actions,
        }
    }

    /// Handles the user provided event.
    pub fn handle_event(&mut self, event: Event) -> Option<GUIEvent> {
        if let Event::Input(input) = event {
            self.selected = match input {
                // Move up.
                keyevent::Key::Char('k') => self.selected.max(1) - 1,
                // Move down.
                keyevent::Key::Char('j') => (self.selected + 1).min(self.actions.len() - 1),
                _ => self.selected,
            };
            return match input {
                keyevent::Key::Char('\n') => match self.actions[self.selected] {
                    Action::NewGame => {
                        let confirm_action =
                            Box::new(|_: &mut Sender<Event>| Some(GUIEvent::StartNewGame));

                        let cancel_action =
                            Box::new(|_: &mut Sender<Event>| Some(GUIEvent::CloseDialog));
                        Some(GUIEvent::OpenDialog(Box::new(ConfirmDialog::new(
                            String::from("Are you sure? All saves will be lost"),
                            confirm_action,
                            cancel_action,
                        ))))
                    }
                    Action::LoadGame => Some(GUIEvent::LoadExistingGame),
                },
                _ => None,
            };
        }
        None
    }

    /// Draws the dialog in the given terminal and area.
    pub fn draw(&self, term: &mut Terminal<MouseBackend>, area: Rect) {
        let page_area = Rect::new((area.width - 80) / 2, (area.height - 40) / 2, 80, 40);
        let labels: Vec<String> = self
            .actions
            .iter()
            .map(|a| format!("{:^1$}", a.to_string(), 80))
            .collect::<Vec<_>>();
        Group::default()
            .direction(Direction::Vertical)
            .sizes(&[Size::Percent(50), Size::Percent(30)])
            .render(term, &page_area, |term, chunks| {
                Paragraph::default()
                    .block(Block::default())
                    .style(Style::default().fg(Color::Green))
                    .alignment(Alignment::Center)
                    .text(&self.title)
                    .render(term, &chunks[0]);
                SelectableList::default()
                    .items(&labels)
                    .select(self.selected)
                    .block(Block::default())
                    .style(Style::default().fg(Color::Green))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .render(term, &chunks[1]);
            });
    }
}
