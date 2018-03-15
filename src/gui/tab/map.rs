use super::*;
use std::collections::HashMap;
use termion::event as keyevent;

use nalgebra::{distance, Vector2};
use utils::Point;
use entities::Faction;
use astronomicals::system::System;
use tui::widgets::{Block, Borders, Row, SelectableList, Table, Widget};
use tui::widgets::canvas::Canvas;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};

lazy_static! {
    /// Color mapping for each faction.
    static ref FACTION_COLORS: HashMap<Faction, Color> = {
        let mut m = HashMap::new();
        m.insert(Faction::Empire, Color::Red);
        m.insert(Faction::Federation, Color::Yellow);
        m.insert(Faction::Cartel, Color::Magenta);
        m.insert(Faction::Independent, Color::LightGreen);
        m
    };

    /// Styling for unselected item.
    static ref DEFAULT_STYLE: Style = Style::default();
}

/// The minimum distance within which the gui will snap to the closest system.
const MIN_SNAP_DIST: f64 = 0.9;

/// Displays the map tab.
pub struct MapTab {
    state: Arc<Game>,
    sender: Sender<Event>,
    selected: Option<Point>,
    cursor: Point,
    map_scale: f64,
}

impl MapTab {
    /// Moves the player's location to the selected system.
    fn travel_to_selected(&self) {
        self.state
            .player
            .lock()
            .unwrap()
            .set_location(&self.selected.unwrap());
        self.sender.send(Event::Travel);
    }
}

impl Tab for MapTab {
    /// Creates a map tab.
    fn new(state: Arc<Game>, send_handle: Sender<Event>) -> Box<Self> {
        let cursor = state.player.lock().unwrap().location().clone();
        Box::new(MapTab {
            state: state,
            sender: send_handle,
            selected: Some(cursor.clone()),
            cursor: cursor,
            map_scale: 1.,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Galaxy Map")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Input(input) => {
                match input {
                    keyevent::Key::Char(' ') if self.selected.is_some() => {
                        self.travel_to_selected()
                    }
                    _ => {}
                };

                self.map_scale *= match input {
                    // Zoom out.
                    keyevent::Key::Char('u') => 0.5,
                    // Zoom in.
                    keyevent::Key::Char('i') => 2.,

                    // No zooming.
                    _ => 1.,
                };

                // Prevent zooming too far in.
                self.map_scale = self.map_scale.min(4.);

                let translation = match input {
                    // Move up.
                    keyevent::Key::Char('k') => Vector2::new(0., 1. / self.map_scale),
                    // Move down.
                    keyevent::Key::Char('j') => Vector2::new(0., -1. / self.map_scale),
                    // Move right.
                    keyevent::Key::Char('l') => Vector2::new(1. / self.map_scale, 0.),
                    // Move left.
                    keyevent::Key::Char('h') => Vector2::new(-1. / self.map_scale, 0.),
                    _ => Vector2::new(0., 0.),
                };

                // Move out of snapping system if currently snapped.
                self.cursor += match self.selected {
                    Some(_) => Vector2::new(
                        translation.x * 1.1 * self.map_scale * MIN_SNAP_DIST,
                        translation.y * 1.1 * self.map_scale * MIN_SNAP_DIST,
                    ),
                    None => translation,
                };
                self.selected = None;

                // Check if cursor should snap to closest system.
                if let Some(neighbor) = self.state.galaxy.lock().unwrap().nearest(&self.cursor) {
                    if distance(&self.cursor, &neighbor) < MIN_SNAP_DIST {
                        self.cursor = neighbor.clone();
                        self.selected = Some(neighbor.clone());
                    }
                }
            }
            _ => {}
        };
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        let galaxy = self.state.galaxy.lock().unwrap();
        Group::default()
            .direction(Direction::Horizontal)
            .sizes(&[Size::Fixed(70), Size::Min(1)])
            .render(term, area, |term, chunks| {
                // TODO: Draw system detailed information.
                let systems = &galaxy.systems();
                match self.selected {
                    Some(point) => {
                        draw_system_info(galaxy.system(&point).unwrap(), term, &chunks[0])
                    }
                    _ => {}
                }
                draw_galaxy_map(&self.cursor, &systems, &self.map_scale, term, &chunks[1]);
            });
    }
}

/// Draw system ship information for the selected system, if any.
fn draw_system_info(selected_system: &System, term: &mut Terminal<MouseBackend>, area: &Rect) {
    let system_data = vec![
        format!("Faction:   {}", selected_system.faction.to_string()),
        format!("State:     {}", selected_system.state.to_string()),
        format!("Star mass: {} M", selected_system.star.mass),
        format!("Bodies:    {}", selected_system.satelites.len()),
    ];

    Group::default()
        .direction(Direction::Vertical)
        .sizes(&[Size::Fixed(6), Size::Min(1)])
        .render(term, area, |term, chunks| {
            SelectableList::default()
                .items(&system_data)
                .block(Block::default().title(selected_system.name.clone().as_str()))
                .style(Style::default().fg(Color::Yellow))
                .render(term, &chunks[0]);
            Table::new(
                // Prepending empty character to get alignment with list above.
                [" Planet", "Mass", "Orbit distance", "Temperature", "Type"].into_iter(),
                selected_system.satelites.iter().map(|ref planet| {
                    let style: &Style = &DEFAULT_STYLE;
                    Row::StyledData(
                        vec![
                            format!(" {}", planet.name.clone()),
                            planet.mass.to_string(),
                            planet.orbit_distance.to_string(),
                            planet.surface_temperature.to_string(),
                            planet.planet_type.to_string(),
                        ].into_iter(),
                        &style,
                    )
                }),
            ).block(Block::default().title("Planets"))
                .header_style(Style::default().fg(Color::Yellow))
                .widths(&[15, 5, 15, 15, 5])
                .render(term, &chunks[1]);
        });
}

fn draw_galaxy_map(
    cursor: &Point,
    systems: &Vec<&System>,
    map_scale: &f64,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    // Scale map to not overlap systems.
    let map_scaling = 20. * map_scale;
    let (max_x, max_y) = systems.iter().fold((0., 0.), |(x_max, y_max), s| {
        (
            (s.location.coords.x / map_scaling).abs().max(x_max),
            (s.location.coords.y / map_scaling).abs().max(y_max),
        )
    });
    Canvas::default()
        .block(Block::default().title("Systems").borders(Borders::ALL))
        .paint(|ctx| {
            for system in systems.iter() {
                let color = FACTION_COLORS.get(&system.faction).unwrap().clone();
                ctx.print(
                    system.location.coords.x,
                    system.location.coords.y,
                    ".",
                    color,
                );
            }
            ctx.print(cursor.coords.x, cursor.coords.y, "*", Color::Yellow);
        })
        .x_bounds([cursor.coords.x - max_x, cursor.coords.x + max_x])
        .y_bounds([cursor.coords.y - max_y, cursor.coords.y + max_y])
        .render(term, &area);
}
