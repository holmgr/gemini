use super::*;
use std::collections::HashMap;
use termion::event as keyevent;

use entities::Faction;
use astronomicals::sector::Sector;
use astronomicals::system::System;
use tui::widgets::{Block, Borders, Row, Table, Widget};
use tui::widgets::canvas::Canvas;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};

/// Level of map display.
enum Level {
    Galaxy,
    Sector,
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
    selected_sector: usize,
    max_selected_sector: usize,
    selected_system: usize,
    max_selected_system: usize,
    selected_astronomical: usize,
    max_selected_astronomical: usize,
}

impl Tab for MapTab {
    /// Creates a map tab.
    fn new(state: Arc<Game>) -> Box<Self> {
        let max_selected_sector = state.galaxy.lock().unwrap().sectors.len();

        Box::new(MapTab {
            state: state,
            level: Level::Galaxy,
            selected_sector: 0,
            max_selected_sector: max_selected_sector,
            selected_system: 0,
            max_selected_system: 0,
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
                        self.selected_sector = (self.selected_sector as i32 - 1).max(0) as usize
                    }
                    Level::Sector => {
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
                        self.selected_sector =
                            (self.selected_sector + 1).min(self.max_selected_sector)
                    }
                    Level::Sector => {
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
                            self.max_selected_system = self.state.galaxy.lock().unwrap().sectors
                                [self.selected_sector]
                                .systems
                                .len();
                            Level::Sector
                        }
                        Level::Sector => {
                            self.max_selected_astronomical =
                                self.state.galaxy.lock().unwrap().sectors[self.selected_sector]
                                    .systems[self.selected_system]
                                    .satelites
                                    .len();
                            Level::System
                        }
                        _ => Level::Galaxy,
                    };
                }

                // Move out of item.
                keyevent::Key::Char('h') => {
                    self.level = match self.level {
                        Level::System => Level::Sector,
                        Level::Sector => Level::Galaxy,
                        _ => Level::Galaxy,
                    };
                }
                _ => {}
            },
            _ => {}
        };

        // Update sizes of lists
        let galaxy = &self.state.galaxy.lock().unwrap();
        self.max_selected_sector = galaxy.sectors.len();
        self.max_selected_system = galaxy.sectors[self.selected_sector].systems.len();
        self.max_selected_astronomical = galaxy.sectors[self.selected_sector].systems
            [self.selected_system]
            .satelites
            .len();
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        Group::default()
            .direction(Direction::Horizontal)
            .sizes(&[Size::Percent(30), Size::Percent(70)])
            .render(term, area, |term, chunks| match self.level {
                Level::Galaxy => {
                    let galaxy = self.state.galaxy.lock().unwrap();
                    draw_galaxy_table(self.selected_sector, &galaxy.sectors, term, &chunks[0]);
                    draw_galaxy_map(self.selected_sector, &galaxy.sectors, term, &chunks[1]);
                }
                Level::Sector => {
                    let sector = &self.state.galaxy.lock().unwrap().sectors[self.selected_sector];
                    draw_sector_table(self.selected_system, &sector, term, &chunks[0]);
                    draw_sector_map(self.selected_system, &sector, term, &chunks[1]);
                }
                Level::System => {
                    let system = &self.state.galaxy.lock().unwrap().sectors[self.selected_sector]
                        .systems[self.selected_system];
                    draw_system_table(self.selected_astronomical, &system, term, &chunks[0]);
                    draw_system_map(self.selected_astronomical, &system, term, &chunks[1]);
                }
            });
    }
}

fn draw_galaxy_table(
    selected: usize,
    sectors: &Vec<Sector>,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    Table::new(
        ["Sector", "Faction", "Systems"].into_iter(),
        sectors
            .iter()
            .enumerate()
            .map(|(idx, ref sector)| {
                let style = match selected == idx {
                    true => &SELECTED_STYLE,
                    false => FACTION_STYLES.get(&sector.faction).unwrap(),
                };
                Row::StyledData(
                    vec![
                        sector.name.clone(),
                        sector.faction.to_string(),
                        (sector.systems.len() as u32).to_string(),
                    ].into_iter(),
                    style,
                )
            })
            .skip(selected),
    ).block(Block::default().title("Sectors").borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(&[30, 15, 5])
        .render(term, &area);
}

fn draw_galaxy_map(
    selected: usize,
    sectors: &Vec<Sector>,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    let (max_x, max_y) = sectors.iter().fold((0., 0.), |(x_max, y_max), s| {
        let location = s.center();
        (
            location.coords.x.abs().max(x_max),
            location.coords.y.abs().max(y_max),
        )
    });
    let selected_loc = sectors[selected].center();
    Canvas::default()
        .block(Block::default().title("Galaxy").borders(Borders::ALL))
        .paint(|ctx| {
            for (idx, sector) in sectors.iter().enumerate() {
                let color = match sector.faction {
                    Faction::Empire => Color::Red,
                    Faction::Federation => Color::Yellow,
                    Faction::Cartel => Color::Magenta,
                    Faction::Independent => Color::LightGreen,
                };
                let center = sector.center();
                match idx == selected {
                    true => ctx.print(center.coords.x, center.coords.y, "X", color),
                    false => ctx.print(center.coords.x, center.coords.y, "*", color),
                };
            }
        })
        .x_bounds([
            -max_x + selected_loc.coords.x,
            selected_loc.coords.x + max_x,
        ])
        .y_bounds([
            -max_y + selected_loc.coords.y,
            selected_loc.coords.y + max_y,
        ])
        .render(term, area);
}

fn draw_sector_table(
    selected: usize,
    sector: &Sector,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    Table::new(
        ["System", "Bodies"].into_iter(),
        sector
            .systems
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
    ).block(Block::default().title(&sector.name).borders(Borders::ALL))
        .header_style(Style::default().fg(Color::Yellow))
        .widths(&[30, 15, 5])
        .render(term, &area);
}

fn draw_sector_map(
    selected: usize,
    sector: &Sector,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    let sector_center = sector.center();
    let (max_x, max_y) = sector.systems.iter().fold((0., 0.), |(x_max, y_max), s| {
        (
            (sector_center.coords.x - s.location.coords.x)
                .abs()
                .max(x_max),
            (sector_center.coords.y - s.location.coords.y)
                .abs()
                .max(y_max),
        )
    });
    let selected_loc = sector.systems[selected].location;
    Canvas::default()
        .block(Block::default().title("Sector").borders(Borders::ALL))
        .paint(|ctx| {
            for (idx, system) in sector.systems.iter().enumerate() {
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
