use super::*;
use std::collections::HashMap;
use termion::event as keyevent;
use tui::{
    layout::{Direction, Group, Rect, Size},
    style::{Color, Style},
    widgets::{canvas::Canvas, Block, Borders, Paragraph, Row, SelectableList, Table, Widget},
};

use super::GUIEvent;
use astronomicals::System;
use entities::Faction;
use player::Player;
use utils::Point;

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

/// Displays the galaxy map tab.
pub struct GalaxyMapTab {
    state: Arc<Game>,
    sender: Sender<Event>,
    search_mode: bool,
    search_str: String,
    selected: Option<Point>,
    route: Option<(u32, Vec<Point>)>,
    cursor: Point,
    map_scale: f64,
}

impl GalaxyMapTab {
    /// Attempts to find a route to the selected system.
    fn find_route(&mut self) {
        let galaxy = &self.state.galaxy.lock().unwrap();
        let player = &mut self.state.player.lock().unwrap();
        let range = match *player.ship() {
            Some(ref ship) => ship.range(),
            None => 0.,
        };
        let max_jumps = match *player.ship() {
            Some(ref ship) => ship.fuel(),
            None => 0,
        };
        // Plan route if possible.
        self.route = galaxy.route(player.location(), self.selected.unwrap(), range, max_jumps);
    }

    /// Moves the player's location to the selected system.
    fn travel_to_selected(&mut self) {
        let player = &mut self.state.player.lock().unwrap();

        // Only travel if the selected system is the same as the cursor and
        // and the final destination for the route.
        if let Some((_, ref route)) = self.route {
            if self.selected.is_some()
                && self.selected.unwrap() == self.cursor
                && self.selected.unwrap() == *route.last().unwrap()
            {
                player.set_route(route.clone());
                self.sender.send(Event::Travel).unwrap();
            }
        }

        // Reset route.
        self.route = None;
    }

    /// Draws the event box in the given terminal and area.
    pub fn draw_search(&self, term: &mut Terminal<MouseBackend>, area: Rect) {
        let draw_str = if self.search_mode {
            format!("{}{}", self.search_str, "{mod=bold |}")
        } else {
            String::from("Press '/' to search for a system")
        };
        Paragraph::default()
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow))
            .text(draw_str.as_str())
            .render(term, &area);
    }

    /// Draw system ship information for the selected system, if any.
    fn draw_system_info(
        &self,
        player_loc: &Point,
        selected_system: Option<&System>,
        term: &mut Terminal<MouseBackend>,
        area: Rect,
    ) {
        // Do not draw anything if no system is selected.
        if selected_system.is_none() {
            return;
        }
        let system = selected_system.unwrap();

        let system_data = vec![
            format!("Faction:       {}", system.faction.to_string()),
            format!("State:         {}", system.state.to_string()),
            format!("Security:      {}", system.security.to_string()),
            format!("Reputation:    {}", system.reputation.to_string()),
            format!(
                "Distance:      {:.1} ly",
                player_loc.distance(&system.location)
            ),
            format!("Star mass:     {:.1} M", system.star.mass),
            format!("Star type:     {}", system.star.startype.to_string()),
            format!("Bodies:        {}", system.satelites.len()),
        ];

        Group::default()
            .direction(Direction::Vertical)
            .sizes(&[Size::Fixed(9), Size::Min(1)])
            .render(term, &area, |term, chunks| {
                SelectableList::default()
                    .items(&system_data)
                    .block(Block::default().title(&format!("{} System", system.name)))
                    .style(Style::default().fg(Color::Yellow))
                    .render(term, &chunks[0]);
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
                        .map(|planet| {
                            let style: &Style = &DEFAULT_STYLE;

                            // Calculate the cardinal populaton name.
                            let cardinal_population = match planet.population {
                                 99999...999_999_999 => format!("{:.1} M", planet.population as f64 / 1_000_000.),
                                 999_999_999...999_999_999_999 => format!("{:.1} B", planet.population as f64 / 1_000_000_000.),
                                 _ => format!("{:.1}", planet.population),
                            };
                            Row::StyledData(
                                vec![
                                    format!(" {}", planet.name.clone()),
                                    format!("{:.1}", planet.mass),
                                    cardinal_population,
                                    format!("{:.1}", planet.surface_temperature),
                                    planet.planet_type.to_string(),
                                    planet.economic_type.to_string(),
                                ].into_iter(),
                                &style,
                            )
                        }),
                ).block(Block::default().title("Planets"))
                    .header_style(Style::default().fg(Color::Yellow))
                    .widths(&[15, 5, 15, 15, 10, 15])
                    .render(term, &chunks[1]);
            });
    }

    /// Draw the galaxy map.
    fn draw_galaxy_map(
        &self,
        player: &Player,
        systems: &[&System],
        map_radius: f64,
        term: &mut Terminal<MouseBackend>,
        area: Rect,
    ) {
        let player_loc = player.location();
        Canvas::default()
            .block(Block::default().title("Systems").borders(Borders::ALL))
            .paint(|ctx| {
                for system in systems.iter() {
                    let color = *FACTION_COLORS.get(&system.faction).unwrap();
                    ctx.print(system.location.x, system.location.y, ".", color);
                }
                // Draw player location.
                ctx.print(player_loc.x, player_loc.y, "X", Color::White);

                // Draw the cursor.
                ctx.print(self.cursor.x, self.cursor.y, "*", Color::Yellow);

                // Draw route if available.
                if let Some((_, ref route)) = self.route {
                    for system in route {
                        ctx.print(system.x, system.y, "X", Color::Yellow);
                    }
                    ctx.print(player_loc.x, player_loc.y, "S", Color::Yellow);
                    ctx.print(
                        route.last().unwrap().x,
                        route.last().unwrap().y,
                        "G",
                        Color::Yellow,
                    );
                }
                // Draw currently travelling route if available.
                if let Some(ref route) = player.route() {
                    for system in route {
                        ctx.print(system.x, system.y, "X", Color::White);
                    }
                    ctx.print(player_loc.x, player_loc.y, "S", Color::White);
                    ctx.print(
                        route.last().unwrap().x,
                        route.last().unwrap().y,
                        "G",
                        Color::White,
                    );
                }
            })
            .x_bounds([self.cursor.x - map_radius, self.cursor.x + map_radius])
            .y_bounds([self.cursor.y - map_radius, self.cursor.y + map_radius])
            .render(term, &area);
    }
}

impl Tab for GalaxyMapTab {
    /// Creates a map tab.
    fn new(state: Arc<Game>, send_handle: Sender<Event>) -> Box<Self> {
        let cursor = state.player.lock().unwrap().location();
        Box::new(GalaxyMapTab {
            state,
            sender: send_handle,
            selected: Some(cursor),
            search_mode: false,
            search_str: String::new(),
            route: None,
            cursor,
            map_scale: 1.,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Galaxy Map")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) -> Option<GUIEvent> {
        if let Event::Input(input) = event {
            match input {
                keyevent::Key::Char('\n') if self.search_mode => {
                    let galaxy = self.state.galaxy.lock().unwrap();

                    // Set cursor to the closest matching system if
                    // possible.
                    if let Some(system) = galaxy.search_name(&self.search_str) {
                        self.cursor = system.location;
                    };

                    // Clear input.
                    self.search_str.clear();
                    self.search_mode = false;
                }
                keyevent::Key::Char(e) if self.search_mode => {
                    self.search_str.push(e);
                    // Early exit.
                    return None;
                }
                keyevent::Key::Backspace if self.search_mode => {
                    self.search_str.pop();
                }
                keyevent::Key::Char('\n') if self.selected.is_some() => {
                    match self.route {
                        Some(_) => self.travel_to_selected(),
                        None => self.find_route(),
                    };
                }
                // Center map around player
                keyevent::Key::Char(' ') => {
                    if let Ok(player) = self.state.player.lock() {
                        self.cursor = player.location();
                    }
                }
                // Start search mode.
                keyevent::Key::Char('/') => {
                    self.search_mode = true;
                    return None;
                }
                // Quit search mode.
                keyevent::Key::Esc => {
                    self.search_str.clear();
                    self.search_mode = false;
                    return None;
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
                keyevent::Key::Char('k') => Point::new(0., 1. / self.map_scale),
                // Move down.
                keyevent::Key::Char('j') => Point::new(0., -1. / self.map_scale),
                // Move right.
                keyevent::Key::Char('l') => Point::new(1. / self.map_scale, 0.),
                // Move left.
                keyevent::Key::Char('h') => Point::new(-1. / self.map_scale, 0.),
                _ => Point::new(0., 0.),
            };

            // Move out of snapping system if currently snapped.
            self.cursor += match self.selected {
                Some(_) => Point::new(
                    translation.x * 1.1 * self.map_scale * MIN_SNAP_DIST,
                    translation.y * 1.1 * self.map_scale * MIN_SNAP_DIST,
                ),
                None => translation,
            };
            self.selected = None;

            // Check if cursor should snap to closest system.
            if let Some(neighbor) = self.state.galaxy.lock().unwrap().nearest(&self.cursor) {
                if self.cursor.distance(&neighbor) < MIN_SNAP_DIST {
                    self.cursor = *neighbor;
                    self.selected = Some(*neighbor);
                }
            }
        }
        None
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        let galaxy = self.state.galaxy.lock().unwrap();
        Group::default()
            .direction(Direction::Horizontal)
            .sizes(&[Size::Fixed(85), Size::Min(1)])
            .render(term, area, |term, chunks| {
                // TODO: Draw system detailed information.
                let player = &self.state.player.lock().unwrap();

                // Draw sidebar.
                Group::default()
                    .direction(Direction::Vertical)
                    .sizes(&[Size::Min(1), Size::Fixed(3)])
                    .render(term, &chunks[0], |term, sidebar_chunk| {
                        self.draw_system_info(
                            &player.location(),
                            self.selected.map(|point| galaxy.system(&point).unwrap()),
                            term,
                            sidebar_chunk[0],
                        );
                        self.draw_search(term, sidebar_chunk[1]);
                    });

                // Find all systems which should be drawn on screen
                let map_radius = 20. / self.map_scale;
                let upper_left = self.cursor + Point::new(-map_radius, -map_radius);
                let lower_right = self.cursor + Point::new(map_radius, map_radius);

                let systems = galaxy
                    .reachable_rect(&upper_left, &lower_right)
                    .into_iter()
                    .map(|loc| galaxy.system(&loc).unwrap())
                    .collect::<Vec<_>>();
                self.draw_galaxy_map(player, &systems, map_radius, term, chunks[1]);
            });
    }
}
