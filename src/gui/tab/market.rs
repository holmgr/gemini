use super::*;
use player::PlayerState;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Row, Table, Widget};

lazy_static! {
    /// Styling for selected item.
    static ref SELECTED_STYLE: Style = Style::default().fg(Color::Yellow);

    /// Styling for unselected item.
    static ref DEFAULT_STYLE: Style = Style::default();
}

/// Displays the market tab.
pub struct MarketTab {
    selected: usize,
    state: Arc<Game>,
    sender: Sender<Event>,
}

impl Tab for MarketTab {
    /// Creates a market tab.
    fn new(state: Arc<Game>, send_handle: Sender<Event>) -> Box<Self> {
        Box::new(MarketTab {
            selected: 0,
            state,
            sender: send_handle,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Market")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, _event: Event) {}

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        let player = self.state.player.lock().unwrap();

        match player.state() {
            PlayerState::Docked(_) => {
                let galaxy = self.state.galaxy.lock().unwrap();
                let system = galaxy.system(player.location()).unwrap();
                let prices = self.state.economy.lock().unwrap().commodity_prices(system);

                Table::new(
                    // Prepending empty character to get alignment with list above.
                    ["Commodity", "Buy", "Sell"].into_iter(),
                    prices.iter().map(|(commodity, price)| {
                        let style = &DEFAULT_STYLE;
                        Row::StyledData(
                            vec![
                                format!("{}", commodity.to_string()),
                                format!("{:.1}", price),
                                format!("{:.1}", (*price as f64 * 0.8) as i64),
                            ].into_iter(),
                            &style,
                        )
                    }),
                ).block(Block::default().title("Commodities").borders(Borders::ALL))
                    .header_style(Style::default().fg(Color::Yellow))
                    .widths(&[40, 50, 50])
                    .render(term, &area);
            }
            _ => {}
        }
    }
}
