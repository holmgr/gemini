use super::*;
use termion::event as keyevent;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Row, Table, Widget};

use super::GUIEvent;
use player::PlayerState;

lazy_static! {
    /// Styling for selected item.
    static ref SELECTED_STYLE: Style = Style::default().fg(Color::Yellow);

    /// Styling for unselected item.
    static ref DEFAULT_STYLE: Style = Style::default();
}

/// Displays the market tab.
pub struct MarketTab {
    selected: usize,
    max_selected: usize,
    state: Arc<Game>,
    sender: Sender<Event>,
}

impl Tab for MarketTab {
    /// Creates a market tab.
    fn new(state: Arc<Game>, send_handle: Sender<Event>) -> Box<Self> {
        let cloned_state = state.clone();
        let galaxy = state.galaxy.lock().unwrap();
        let system = galaxy
            .system(&state.player.lock().unwrap().location())
            .unwrap();
        let max_selected = state.economy.lock().unwrap().commodity_prices(system).len() - 1;

        Box::new(MarketTab {
            selected: 0,
            max_selected,
            state: cloned_state,
            sender: send_handle,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Market")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, event: Event) -> Option<GUIEvent> {
        match event {
            Event::Input(input) => {
                // TODO: Open dialog with sell/buy of goods.
                if let keyevent::Key::Char('\n') = input {}
                self.selected = match input {
                    // Move up.
                    keyevent::Key::Char('k') => self.selected.max(1) - 1,
                    // Move down.
                    keyevent::Key::Char('j') => (self.selected + 1).min(self.max_selected),
                    _ => self.selected,
                };
            }
            Event::Update => {
                // Update maximum index if needed.
                let galaxy = self.state.galaxy.lock().unwrap();
                let system = galaxy
                    .system(&self.state.player.lock().unwrap().location())
                    .unwrap();
                self.max_selected = self
                    .state
                    .economy
                    .lock()
                    .unwrap()
                    .commodity_prices(system)
                    .len() - 1;
                self.selected = self.selected.min(self.max_selected);
            }
            _ => {}
        };
        None
    }

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        let player = self.state.player.lock().unwrap();

        if let PlayerState::Docked(_) = player.state() {
            let galaxy = self.state.galaxy.lock().unwrap();
            let system = galaxy.system(&player.location()).unwrap();
            let prices = self.state.economy.lock().unwrap().commodity_prices(system);

            Table::new(
                ["Commodity", "Buy", "Sell"].into_iter(),
                prices.iter().enumerate().map(|(idx, (commodity, price))| {
                    let style: &Style = if idx == self.selected {
                        &SELECTED_STYLE
                    } else {
                        &DEFAULT_STYLE
                    };
                    Row::StyledData(
                        vec![
                            commodity.to_string(),
                            format!("{:.1}", price),
                            format!("{:.1}", (*price as f64 * 0.8) as i64),
                        ].into_iter(),
                        &style,
                    )
                }),
            ).block(Block::default().title("Commodities").borders(Borders::ALL))
                .header_style(Style::default().fg(Color::Yellow))
                .widths(&[40, 50, 50, 90])
                .render(term, &area);
        }
    }
}
