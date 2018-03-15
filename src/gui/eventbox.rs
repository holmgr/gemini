use tui::Terminal;
use tui::backend::MouseBackend;
use tui::widgets::{Block, Borders, Paragraph, Widget};
use tui::layout::Rect;
use tui::style::{Color, Style};
use std::time::Instant;

use event::Event;

/// Displays game information messages like autosaving completions to user.
pub struct EventBox {
    message: String,
    lastevent: Instant,
}

impl EventBox {
    /// Time until message is cleared.
    const EVENT_TIMEOUT: u64 = 2;

    pub fn new() -> Self {
        EventBox {
            message: String::new(),
            lastevent: Instant::now(),
        }
    }

    /// Handles the user provided event.
    pub fn handle_event(&mut self, event: Event) {
        let new_message = match event {
            Event::AutosaveStarted => Some(String::from("Autosave in progress...")),
            Event::AutosaveCompleted => Some(String::from("Autosave completed!")),
            _ => None,
        };

        // Update if new message available.
        if new_message.is_some() {
            // Reset timer for last event.
            self.lastevent = Instant::now();
            self.message = new_message.unwrap();
        }
    }

    /// Draws the event box in the given terminal and area.
    pub fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        // Only draw if last event is recent.
        let msg = match self.lastevent.elapsed().as_secs() < EventBox::EVENT_TIMEOUT {
            true => format!("$ {}", self.message.as_str()),
            false => String::from("$ "),
        };
        Paragraph::default()
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow))
            .text(msg.as_str())
            .render(term, &area);
    }
}
