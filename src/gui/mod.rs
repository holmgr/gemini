use std::{io, sync::Arc};
use termion::event as keyevent;
use tui::{
    backend::MouseBackend,
    layout::{Direction, Group, Rect, Size},
    style::{Color, Style},
    widgets::{Block, Borders, Tabs, Widget},
    Terminal,
};

use event::{add_keyboard_handler, add_player_handler, add_update_handler, Event, HANDLER};
use game::Game;

pub mod dialog;
mod info;
mod tab;
mod title;

use self::dialog::Dialog;
use self::info::draw_info_page;
use self::title::TitlePage;
use simulator::Simulator;

/// Events used for communicating specifically between GUI components.
pub enum GUIEvent {
    StartNewGame,
    LoadExistingGame,
    OpenDialog(Box<dyn Dialog>),
    CloseDialog,
}

/// Handles the graphical user interface to the user.
pub struct Gui {
    simulator: Simulator,
    size: Rect,
    tabs: Vec<Box<dyn tab::Tab>>,
    selected_tab: usize,
    dialog: Option<Box<dyn Dialog>>,
    title_page: Option<TitlePage>,
}

impl Gui {
    /// Creates a new GUI
    pub fn new(simulator: Simulator) -> Self {
        // TODO: Make a bit more elegant
        add_keyboard_handler();

        Gui {
            simulator,
            size: Rect::default(),
            tabs: vec![], //tab::create_tabs(&game),
            selected_tab: 0,
            dialog: None,
            title_page: Some(TitlePage::new()),
        }
    }

    fn start_main_game(&mut self, game_state: &Arc<Game>) {
        // TODO: Move to some where more reasonable.
        add_player_handler(game_state.clone());
        // TODO: Move to some where more reasonable.
        add_update_handler(game_state.clone());

        // Initialize all tabs.
        self.tabs = tab::create_tabs(game_state);

        // Disable title screen and initial dialog window if any.
        self.title_page = None;
        self.dialog = None;
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
                        // Forward event to current tab or dialog/title page if open.
                        let gui_event = match (&mut self.dialog, &mut self.title_page) {
                            (Some(ref mut dialog), _) => dialog.handle_event(evt),
                            (None, Some(ref mut title_page)) => title_page.handle_event(evt),
                            _ => self.tabs[self.selected_tab].handle_event(evt),
                        };
                        match gui_event {
                            Some(GUIEvent::StartNewGame) => {
                                // Draw loading screen.
                                draw_info_page(
                                    &mut term,
                                    self.size,
                                    "Creating a new galaxy so hang in there...",
                                );

                                // Ensure that the screen gets redrawn.
                                term.draw().unwrap();

                                let game_state = self.simulator.new_game();
                                self.start_main_game(&game_state);
                            }
                            Some(GUIEvent::LoadExistingGame) => {
                                if let Some(game_state) = self.simulator.load_game() {
                                    self.start_main_game(&game_state);
                                } else {
                                    // Show basic feedback to user about failing to load.
                                    self.dialog = Some(Box::new(dialog::AlertDialog::new(
                                        String::from("No game save found"),
                                        Box::new(|_| Some(GUIEvent::CloseDialog)),
                                    )));
                                }
                            }
                            Some(GUIEvent::OpenDialog(dialog)) => {
                                debug!("Got back dialog window");
                                self.dialog = Some(dialog);
                            }
                            Some(GUIEvent::CloseDialog) => {
                                self.dialog = None;
                            }
                            _ => {}
                        };
                    }
                },
                _ => {
                    // Forward all general events to all tabs.
                    for tab in &mut self.tabs {
                        tab.handle_event(evt);
                    }
                }
            }
        }
        term.show_cursor().unwrap();
        term.clear().unwrap();
    }

    /// Draws the interface to the terminal.
    fn draw(&self, term: &mut Terminal<MouseBackend>) -> Result<(), io::Error> {
        match (&self.dialog, &self.title_page) {
            (Some(ref dialog), _) => dialog.draw(term, &self.size),
            (None, Some(ref title_page)) => title_page.draw(term, self.size),
            _ => {
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
                        // Draw dialog or current tab.
                        match self.dialog {
                            Some(ref dialog) => dialog.draw(term, &chunks[1]),
                            None => self.tabs[self.selected_tab].draw(term, &chunks[1]),
                        }
                    });
            }
        };
        term.draw()?;
        Ok(())
    }
}
