use super::*;
use termion::event as keyevent;

use astronomicals::System;
use player::PlayerState;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Row, Table, Widget};

use gui::dialog::MultiDialog;

lazy_static! {
    /// Styling for selected item.
    static ref SELECTED_STYLE: Style = Style::default().fg(Color::Yellow);

    /// Styling for selected item.
    static ref DOCKED_STYLE: Style = Style::default().fg(Color::Green);

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
            PlayerState::InSystem | PlayerState::Docked(_) => {
                let galaxy = state.galaxy.lock().unwrap();
                galaxy.system(&player.location()).unwrap().satelites.len() - 1
            }
            _ => 0,
        }
    }

    /// Opens dialog for planet interaction.
    /// Actions available depends on the current player state.
    fn try_open_dialog(&self) -> Option<Box<MultiDialog>> {
        let player = self.state.player.lock().unwrap();
        let galaxy = self.state.galaxy.lock().unwrap();
        let system = galaxy.system(&player.location()).unwrap();
        let planet_id = self.selected_astronomical;

        match player.state() {
            PlayerState::InSystem => {
                // If in system we can dock.
                let dock_fn = Box::new(move || Event::Dock(planet_id));

                Some(MultiDialog::new(
                    system.satelites[self.selected_astronomical].name.clone(),
                    vec![("Dock", dock_fn)],
                ))
            }
            PlayerState::Docked(id) if id == self.selected_astronomical => {
                // If docked system we can refuel.
                let refuel_fn = Box::new(|| Event::Refuel);

                // If docked system we can undock.
                let undock_fn = Box::new(move || Event::Undock(planet_id));

                Some(MultiDialog::new(
                    system.satelites[self.selected_astronomical].name.clone(),
                    vec![("Undock", undock_fn), ("Refuel", refuel_fn)],
                ))
            }
            _ => None,
        }
    }
}

impl Tab for SystemMapTab {
    /// Creates a system map tab.
    fn new(state: Arc<Game>, send_handle: Sender<Event>) -> Box<Self> {
        let max_selected_astronomical = SystemMapTab::num_astronomicals(&state);

        Box::new(SystemMapTab {
            state,
            send_handle,
            selected_astronomical: 0,
            max_selected_astronomical,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("System Map")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) -> Option<GUIEvent> {
        match event {
            Event::Input(input) => {
                // Open planet interaction dialog if appropriate.
                if let keyevent::Key::Char('\n') = input {
                    return self.try_open_dialog()
                        .map(|dialog| GUIEvent::OpenDialog(dialog));
                }
                self.selected_astronomical = match input {
                    // Move up.
                    keyevent::Key::Char('k') => self.selected_astronomical.max(1) - 1,
                    // Move down.
                    keyevent::Key::Char('j') => {
                        (self.selected_astronomical + 1).min(self.max_selected_astronomical)
                    }
                    _ => self.selected_astronomical,
                };
                None
            }
            Event::Update => {
                // Update maximum index if needed.
                self.max_selected_astronomical = SystemMapTab::num_astronomicals(&self.state);
                None
            }
            _ => None,
        }
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        Group::default()
            .direction(Direction::Horizontal)
            .sizes(&[Size::Fixed(85), Size::Percent(70)])
            .render(term, area, |term, chunks| {
                let player = self.state.player.lock().unwrap();
                match player.state() {
                    PlayerState::InSystem => {
                        let galaxy = self.state.galaxy.lock().unwrap();
                        let system = galaxy.system(&player.location()).unwrap();
                        let populations = self.state.economy.lock().unwrap().populations(&system);
                        draw_system_table(
                            self.selected_astronomical,
                            None,
                            &populations,
                            &system,
                            term,
                            chunks[0],
                        );
                        draw_system_map(self.selected_astronomical, &system, term, chunks[1]);
                    }
                    PlayerState::Docked(id) => {
                        let galaxy = self.state.galaxy.lock().unwrap();
                        let system = galaxy.system(&player.location()).unwrap();
                        let populations = self.state.economy.lock().unwrap().populations(&system);
                        draw_system_table(
                            self.selected_astronomical,
                            Some(id),
                            &populations,
                            &system,
                            term,
                            chunks[0],
                        );
                        draw_system_map(self.selected_astronomical, &system, term, chunks[1]);
                    }
                    _ => {}
                }
            });
    }
}

fn draw_system_table(
    selected: usize,
    docked_at: Option<usize>,
    populations: &[f64],
    system: &System,
    term: &mut Terminal<MouseBackend>,
    area: Rect,
) {
    Table::new(
        // Prepending empty character to get alignment with list above.
        [
            " Planet",
            "Mass",
            "Population",
            "Temperature",
            "Type",
            "Economy",
        ].into_iter(),
        system
            .satelites
            .iter()
            .enumerate()
            .map(|(idx, ref planet)| {
                let style: &Style = match docked_at {
                    _ if idx == selected => &SELECTED_STYLE,
                    Some(id) if idx == id => &DOCKED_STYLE,
                    _ => &DEFAULT_STYLE,
                };
                Row::StyledData(
                    vec![
                        format!(" {}", planet.name.clone()),
                        format!("{:.1}", planet.mass),
                        format!("{:.1} M", populations[idx]),
                        format!("{:.1}", planet.surface_temperature),
                        planet.planet_type.to_string(),
                        planet.economic_type.to_string(),
                    ].into_iter(),
                    &style,
                )
            }),
    ).block(Block::default().title(&system.name).borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(&[15, 5, 15, 15, 10, 15])
        .render(term, &area);
}

fn draw_system_map(
    _selected: usize,
    _system: &System,
    _term: &mut Terminal<MouseBackend>,
    _area: Rect,
) {
    // TODO: Find decent presentation of a system, ascii art?
}
