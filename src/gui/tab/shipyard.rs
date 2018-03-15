use super::*;
use std::iter;
use termion::event as keyevent;
use tui::widgets::{Block, Borders, Paragraph, SelectableList, Widget};
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};

use ship::ShipCharacteristics;

/// Displays the shipyard tab.
pub struct ShipyardTab {
    state: Arc<Game>,
    sender: Sender<Event>,
    selected: usize,
    max_selected: usize,
}

impl Tab for ShipyardTab {
    /// Creates a shipyard tab.
    fn new(state: Arc<Game>, send_handle: Sender<Event>) -> Box<Self> {
        let max_selected = &state.shipyard.lock().unwrap().get_available().len();

        Box::new(ShipyardTab {
            state: state,
            sender: send_handle,
            selected: 0,
            max_selected: *max_selected - 1,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Shipyard")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) {
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
            _ => {}
        };

        // Update sizes of lists
        let shipyard = &self.state.shipyard.lock().unwrap();
        self.max_selected = shipyard.get_available().len() - 1;
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        Group::default()
            .direction(Direction::Horizontal)
            //.sizes(&[Size::Percent(10), Size::Percent(90)])
            .sizes(&[Size::Fixed(15), Size::Min(1)])
            .render(term, area, |term, chunks| {
                let ships = &self.state.shipyard.lock().unwrap().get_available().clone();
                draw_ship_list(self.selected, ships, term, &chunks[0]);
                draw_ship_info(&ships[self.selected], term, &chunks[1]);
            });
    }
}

/// Draw a list of the given ships with their names.
fn draw_ship_list(
    selected: usize,
    ships: &Vec<ShipCharacteristics>,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    SelectableList::default()
        .block(Block::default().title("Ships").borders(Borders::ALL))
        .items(&ships
            .iter()
            .map(|ref ship| ship.name.clone())
            .collect::<Vec<String>>())
        .select(selected)
        .style(Style::default())
        .highlight_style(Style::default().bg(Color::White))
        .render(term, &area);
}

/// Draw detailed ship information for a given ship.
fn draw_ship_info(ship: &ShipCharacteristics, term: &mut Terminal<MouseBackend>, area: &Rect) {
    let ship_data = vec![
        ("Name", ship.name.clone()),
        ("Manufacturer", ship.manufacturer.clone()),
        ("Kind", ship.kind.to_string()),
        ("Description", ship.description.clone()),
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
        .render(term, area, |term, chunks| {
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
