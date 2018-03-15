use std::io;
use std::sync::Arc;

use tui::Terminal;
use tui::backend::MouseBackend;
use termion::event as keyevent;
use tui::widgets::{Block, Borders, Tabs, Widget};
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};

mod tab;

use game::Game;
use event::{add_autosave_handler, add_keyboard_handler, Event, HANDLER};

/// Handles the graphical user interface to the user.
pub struct Gui {
    size: Rect,
    tabs: Vec<Box<tab::Tab>>,
    selected_tab: usize,
}

impl Gui {
    /// Creates a new GUI
    pub fn new(game: Arc<Game>) -> Self {
        // TODO: Make a bit more elegant
        add_keyboard_handler();
        // TODO: Move to some where more reasonable.
        add_autosave_handler(game.clone());

        Gui {
            size: Rect::default(),
            tabs: tab::create_tabs(game),
            selected_tab: 0,
        }
    }

    /// Starts the GUI by entering an infinite loop
    pub fn start(&mut self) {
        // Get handle for the user events.
        let rx = HANDLER.recv_handle();

        // Setup terminal interace.
        let backend = MouseBackend::new().unwrap();
        let mut term = Terminal::new(backend).unwrap();
        term.clear().unwrap();
        term.hide_cursor().unwrap();

        loop {
            // Handle resizing.
            let size = term.size().unwrap();
            if size != self.size {
                term.resize(size).unwrap();
                self.size = size;
            }
            self.draw(&mut term).unwrap();

            // Await the next event.
            let evt = rx.recv().unwrap();
            match evt {
                Event::Input(input) => match input {
                    keyevent::Key::Char('q') => {
                        break;
                    }
                    keyevent::Key::Ctrl('h') => {
                        self.selected_tab =
                            (self.tabs.len() + self.selected_tab - 1) % self.tabs.len();
                    }
                    keyevent::Key::Ctrl('l') => {
                        self.selected_tab = (self.selected_tab + 1) % self.tabs.len();
                    }
                    _ => {
                        // Forward event to current tab
                        self.tabs[self.selected_tab].handle_event(evt);
                    }
                },
                _ => {}
            }
        }
        term.show_cursor().unwrap();
        term.clear().unwrap();
    }

    /// Draws the interface to the terminal.
    fn draw(&self, term: &mut Terminal<MouseBackend>) -> Result<(), io::Error> {
        Group::default()
            .direction(Direction::Vertical)
            .sizes(&[Size::Fixed(3), Size::Min(0)])
            .render(term, &self.size, |term, chunks| {
                Tabs::default()
                    .block(Block::default().borders(Borders::ALL).title("Tabs"))
                    .titles(&self.tabs.iter().map(|tab| tab.title()).collect::<Vec<_>>())
                    .style(Style::default().fg(Color::Green))
                    .highlight_style(Style::default().fg(Color::Yellow))
                    .select(self.selected_tab)
                    .render(term, &chunks[0]);
                self.tabs[self.selected_tab].draw(term, &chunks[1]);
            });
        try!(term.draw());
        Ok(())
    }
}
