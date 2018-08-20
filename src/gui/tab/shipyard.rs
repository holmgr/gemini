use super::*;
use std::iter;
use termion::event as keyevent;
use textwrap::fill;
use tui::{
    layout::{Direction, Group, Rect, Size},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, SelectableList, Widget},
};

use ship::ShipCharacteristics;

/// Displays the shipyard tab.
pub struct ShipyardTab {
    state: Arc<Game>,
    sender: Sender<Event>,
    selected: usize,
    max_selected: usize,
    available_ships: Vec<ShipCharacteristics>,
}

impl Tab for ShipyardTab {
    /// Creates a shipyard tab.
    fn new(state: Arc<Game>, send_handle: Sender<Event>) -> Box<Self> {
        // Find all ships available at the current system.
        // If player is not at a system something is very wrong.
        // TODO: Feels bad to clone the arc just to avoid borrower here.
        let dup_state = state.clone();
        let galaxy = dup_state.galaxy.lock().unwrap();
        let player_system = galaxy
            .system(&dup_state.player.lock().unwrap().location())
            .unwrap();
        let available_ships = dup_state
            .shipyard
            .lock()
            .unwrap()
            .get_available(player_system);

        Box::new(ShipyardTab {
            state,
            sender: send_handle,
            selected: 0,
            max_selected: available_ships.len() - 1,
            available_ships,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Shipyard")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) -> Option<GUIEvent> {
        match event {
            Event::Input(input) => match input {
                // Move up item list.
                keyevent::Key::Char('k') => {
                    self.selected = (self.selected as i32 - 1).max(0) as usize
                }
                // Move down item list.
                keyevent::Key::Char('j') => {
                    self.selected = (self.selected + 1).min(self.max_selected)
                }

                _ => {}
            },
            Event::Travel => {
                // Find all ships available at the current system.
                // If player is not at a system something is very wrong.
                let galaxy = self.state.galaxy.lock().unwrap();
                let player_system = galaxy
                    .system(&self.state.player.lock().unwrap().location())
                    .unwrap();
                self.available_ships = self
                    .state
                    .shipyard
                    .lock()
                    .unwrap()
                    .get_available(player_system);
                self.max_selected = self.available_ships.len() - 1;
                // Guard against the number of ships being reduced.
                self.selected = self.selected.min(self.max_selected);
            }
            _ => {}
        };
        None
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        Group::default()
            .direction(Direction::Horizontal)
            //.sizes(&[Size::Percent(10), Size::Percent(90)])
            .sizes(&[Size::Fixed(15), Size::Min(1)])
            .render(term, area, |term, chunks| {
                draw_ship_list(self.selected, &self.available_ships, term, chunks[0]);
                draw_ship_info(&self.available_ships[self.selected], term, chunks[1]);
            });
    }
}

/// Draw a list of the given ships with their names.
fn draw_ship_list(
    selected: usize,
    ships: &[ShipCharacteristics],
    term: &mut Terminal<MouseBackend>,
    area: Rect,
) {
    SelectableList::default()
        .block(Block::default().title("Ships").borders(Borders::ALL))
        .items(
            &ships
                .iter()
                .map(|ref ship| ship.name.clone())
                .collect::<Vec<String>>(),
        )
        .select(selected)
        .style(Style::default())
        .highlight_style(Style::default().bg(Color::White))
        .render(term, &area);
}

/// Draw detailed ship information for a given ship.
fn draw_ship_info(ship: &ShipCharacteristics, term: &mut Terminal<MouseBackend>, area: Rect) {
    let ship_data = vec![
        ("Name", ship.name.clone()),
        ("Manufacturer", ship.manufacturer.clone()),
        ("Kind", ship.kind.to_string()),
        (
            "Description",
            fill(ship.description.as_str(), area.width as usize - 30),
        ),
        ("Cost", ship.cost.to_string()),
        ("Integrity", ship.integrity.to_string()),
        ("Size", ship.size.to_string()),
        ("Mass", ship.mass.to_string()),
        ("Modification slots", ship.slots.to_string()),
        ("Jump range", ship.range.to_string()),
        ("Fuel capacity", ship.fuel.to_string()),
        ("Cargo capacity", ship.cargo.to_string()),
        ("Detectability", ship.detectability.to_string()),
        ("Maneuverability", ship.maneuverability.to_string()),
        ("Defense", ship.defense.to_string()),
        ("Shield", ship.shield.to_string()),
    ];

    // Manually calculate the number of rows needed for the ship description.
    let num_description_rows = ship.description.len() as u16 / area.width + 2;

    let mut row_sizes = iter::repeat(Size::Fixed(2))
        .take(ship_data.len())
        .collect::<Vec<_>>();
    row_sizes[3] = Size::Fixed(num_description_rows);

    Group::default()
        .direction(Direction::Vertical)
        .sizes(row_sizes.as_slice())
        .margin(4)
        .render(term, &area, |term, chunks| {
            for (index, &chunk) in chunks.iter().enumerate() {
                // Draw a single row.
                Group::default()
                    .direction(Direction::Horizontal)
                    .sizes(&[Size::Fixed(20), Size::Min(1)])
                    .render(term, &chunk, |term, inner_chunks| {
                        let (characteristic, ref value) = ship_data[index];
                        Paragraph::default()
                            .block(Block::default())
                            .style(Style::default().fg(Color::White))
                            .wrap(false)
                            .text(characteristic)
                            .render(term, &inner_chunks[0]);
                        Paragraph::default()
                            .block(Block::default())
                            .style(Style::default().fg(Color::White))
                            .wrap(true)
                            .text(value.as_str())
                            .render(term, &inner_chunks[1]);
                    });
            }
        });
}
