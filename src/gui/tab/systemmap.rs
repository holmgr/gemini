use super::*;
use termion::event as keyevent;

use astronomicals::system::System;
use player::PlayerState;
use tui::widgets::{Block, Borders, Row, Table, Widget};
use tui::widgets::canvas::Canvas;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};

lazy_static! {
    /// Styling for selected item.
    static ref SELECTED_STYLE: Style = Style::default().fg(Color::Yellow);

    /// Styling for unselected item.
    static ref DEFAULT_STYLE: Style = Style::default();
}

/// Displays the map tab.
pub struct SystemMapTab {
    state: Arc<Game>,
    send_handle: Sender<Event>,
    selected_astronomical: usize,
    max_selected_astronomical: usize,
}

impl SystemMapTab {
    /// Returns the last index of satelites in the current system, defaults to zero.
    fn num_astronomicals(state: &Arc<Game>) -> usize {
        let player = state.player.lock().unwrap();
        match player.state() {
            PlayerState::Stationary => {
                let galaxy = state.galaxy.lock().unwrap();
                galaxy.system(player.location()).unwrap().satelites.len() - 1
            }
            _ => 0,
        }
    }
}

impl Tab for SystemMapTab {
    /// Creates a system map tab.
    fn new(state: Arc<Game>, send_handle: Sender<Event>) -> Box<Self> {
        let max_selected_astronomical = SystemMapTab::num_astronomicals(&state);

        Box::new(SystemMapTab {
            state: state,
            send_handle: send_handle,
            selected_astronomical: 0,
            max_selected_astronomical: max_selected_astronomical,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("System Map")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Input(input) => {
                self.selected_astronomical = match input {
                    // Move up.
                    keyevent::Key::Char('k') => self.selected_astronomical.max(1) - 1,
                    // Move down.
                    keyevent::Key::Char('j') => {
                        (self.selected_astronomical + 1).min(self.max_selected_astronomical)
                    }
                    _ => self.selected_astronomical,
                };
            }
            Event::Update => {
                // Update maximum index if needed.
                self.max_selected_astronomical = SystemMapTab::num_astronomicals(&self.state);
            }
            _ => {}
        };
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        Group::default()
            .direction(Direction::Horizontal)
            .sizes(&[Size::Fixed(75), Size::Percent(70)])
            .render(term, area, |term, chunks| {
                let player = self.state.player.lock().unwrap();
                match player.state() {
                    PlayerState::Stationary => {
                        let galaxy = self.state.galaxy.lock().unwrap();
                        let system = galaxy.system(player.location()).unwrap();
                        draw_system_table(self.selected_astronomical, &system, term, &chunks[0]);
                        draw_system_map(self.selected_astronomical, &system, term, &chunks[1]);
                    }
                    _ => {}
                }
            });
    }
}

fn draw_system_table(
    selected: usize,
    system: &System,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    Table::new(
        // Prepending empty character to get alignment with list above.
        [" Planet", "Mass", "Population", "Temperature", "Type"].into_iter(),
        system
            .satelites
            .iter()
            .enumerate()
            .map(|(idx, ref planet)| {
                let style: &Style = match selected == idx {
                    true => &SELECTED_STYLE,
                    false => &DEFAULT_STYLE,
                };
                Row::StyledData(
                    vec![
                        format!(" {}", planet.name.clone()),
                        format!("{:.1}", planet.mass),
                        format!("{:.1} M", planet.population),
                        format!("{:.1}", planet.surface_temperature),
                        planet.planet_type.to_string(),
                    ].into_iter(),
                    &style,
                )
            }),
    ).block(Block::default().title(&system.name).borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(&[15, 5, 15, 15, 10])
        .render(term, &area);
}

fn draw_system_map(
    selected: usize,
    system: &System,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    // TODO: Find decent presentation of a system, ascii art?
}
