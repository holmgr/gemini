use super::*;
use std::collections::HashMap;
use termion::event as keyevent;

use utils::Point;
use entities::Faction;
use astronomicals::system::System;
use tui::widgets::{Block, Borders, Row, Table, Widget};
use tui::widgets::canvas::Canvas;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};

/// Level of map display.
enum Level {
    Galaxy,
    System,
}

lazy_static! {
    /// Color mapping for each faction.
    static ref FACTION_STYLES: HashMap<Faction, Style> = {
        let mut m = HashMap::new();
        m.insert(Faction::Empire, Style::default().fg(Color::Red));
        m.insert(Faction::Federation, Style::default().fg(Color::Yellow));
        m.insert(Faction::Cartel, Style::default().fg(Color::Magenta));
        m.insert(Faction::Independent, Style::default().fg(Color::LightGreen));
        m
    };

    /// Styling for selected item.
    static ref SELECTED_STYLE: Style = Style::default().bg(Color::Gray);

    /// Styling for unselected item.
    static ref DEFAULT_STYLE: Style = Style::default();
}

/// Displays the map tab.
pub struct MapTab {
    state: Arc<Game>,
    level: Level,
    selected_system: usize,
    max_selected_system: usize,
    selected_astronomical: usize,
    max_selected_astronomical: usize,
}

impl Tab for MapTab {
    /// Creates a map tab.
    fn new(state: Arc<Game>) -> Box<Self> {
        let max_selected_system = state.galaxy.lock().unwrap().systems().len();

        Box::new(MapTab {
            state: state,
            level: Level::Galaxy,
            selected_system: 0,
            max_selected_system: max_selected_system,
            selected_astronomical: 0,
            max_selected_astronomical: 0,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Galaxy Map")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Input(input) => match input {
                // Move up item list.
                keyevent::Key::Char('k') => match self.level {
                    Level::Galaxy => {
                        self.selected_system = (self.selected_system as i32 - 1).max(0) as usize
                    }
                    Level::System => {
                        self.selected_astronomical =
                            (self.selected_astronomical as i32 - 1).max(0) as usize
                    }
                },

                // Move down item list.
                keyevent::Key::Char('j') => match self.level {
                    Level::Galaxy => {
                        self.selected_system =
                            (self.selected_system + 1).min(self.max_selected_system)
                    }
                    Level::System => {
                        self.selected_astronomical =
                            (self.selected_astronomical + 1).min(self.max_selected_astronomical)
                    }
                },

                // Move into item.
                keyevent::Key::Char('l') => {
                    self.level = match self.level {
                        Level::Galaxy => {
                            self.max_selected_astronomical =
                                self.state.galaxy.lock().unwrap().systems()[self.selected_system]
                                    .satelites
                                    .len();
                            Level::System
                        }
                        _ => Level::System,
                    };
                }

                // Move out of item.
                keyevent::Key::Char('h') => {
                    self.level = match self.level {
                        _ => Level::Galaxy,
                    };
                }
                _ => {}
            },
            _ => {}
        };

        // Update sizes of lists
        let galaxy = &self.state.galaxy.lock().unwrap();
        self.max_selected_system = galaxy.systems.len();
        self.selected_system = self.selected_system.min(self.max_selected_system - 1);
        self.max_selected_astronomical = galaxy.systems()[self.selected_system].satelites.len();
        self.selected_astronomical = self.selected_astronomical
            .min(self.max_selected_astronomical - 1);
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        let galaxy = self.state.galaxy.lock().unwrap();
        Group::default()
            .direction(Direction::Horizontal)
            .sizes(&[Size::Percent(30), Size::Percent(70)])
            .render(term, area, |term, chunks| match self.level {
                Level::Galaxy => {
                    // TODO: Sort based on player location.
                    let systems = &galaxy.systems_ordered(&Point::origin());
                    draw_galaxy_table(self.selected_system, &systems, term, &chunks[0]);
                    draw_galaxy_map(self.selected_system, &systems, term, &chunks[1]);
                }
                Level::System => {
                    let system = &galaxy.systems()[self.selected_system];
                    draw_system_table(self.selected_astronomical, &system, term, &chunks[0]);
                    draw_system_map(self.selected_astronomical, &system, term, &chunks[1]);
                }
            });
    }
}

fn draw_galaxy_table(
    selected: usize,
    systems: &Vec<&System>,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    Table::new(
        ["System", "Bodies"].into_iter(),
        systems
            .iter()
            .enumerate()
            .map(|(idx, ref system)| {
                let style: &Style = match selected == idx {
                    true => &SELECTED_STYLE,
                    false => &DEFAULT_STYLE,
                };
                Row::StyledData(
                    vec![
                        system.name.clone(),
                        (system.satelites.len() as u32).to_string(),
                    ].into_iter(),
                    &style,
                )
            })
            .skip(selected),
    ).block(Block::default().title("System Map").borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(&[30, 15, 5])
        .render(term, &area);
}

fn draw_galaxy_map(
    selected: usize,
    systems: &Vec<&System>,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    // Scale map to not overlap systems.
    let map_scaling = 20.;
    let (max_x, max_y) = systems.iter().fold((0., 0.), |(x_max, y_max), s| {
        (
            (s.location.coords.x / map_scaling).abs().max(x_max),
            (s.location.coords.y / map_scaling).abs().max(y_max),
        )
    });
    let selected_loc = &systems[selected].location;
    Canvas::default()
        .block(Block::default().title("Systems").borders(Borders::ALL))
        .paint(|ctx| {
            for (idx, system) in systems.iter().enumerate() {
                let color = Color::White;
                match idx == selected {
                    true => ctx.print(
                        system.location.coords.x,
                        system.location.coords.y,
                        "X",
                        color,
                    ),
                    false => ctx.print(
                        system.location.coords.x,
                        system.location.coords.y,
                        "*",
                        color,
                    ),
                }
            }
        })
        .x_bounds([selected_loc.coords.x - max_x, selected_loc.coords.x + max_x])
        .y_bounds([selected_loc.coords.y - max_y, selected_loc.coords.y + max_y])
        .render(term, &area);
}

fn draw_system_table(
    selected: usize,
    system: &System,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    Table::new(
        ["Planets", "Mass", "Orbit distance", "Temperature", "Type"].into_iter(),
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
                        planet.name.clone(),
                        planet.mass.to_string(),
                        planet.orbit_distance.to_string(),
                        planet.surface_temperature.to_string(),
                        planet.planet_type.to_string(),
                    ].into_iter(),
                    &style,
                )
            })
            .skip(selected),
    ).block(Block::default().title(&system.name).borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(&[15, 5, 15, 15, 5])
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
