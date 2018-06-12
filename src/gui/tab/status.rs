use super::*;
use tui::{layout::{Direction, Group, Rect, Size},
          style::{Color, Style},
          widgets::{Block, Borders, Paragraph, SelectableList, Widget}};

use player::{Player, PlayerState};

/// Displays the status tab.
pub struct StatusTab {
    state: Arc<Game>,
    sender: Sender<Event>,
}

impl Tab for StatusTab {
    /// Creates a status tab.
    fn new(state: Arc<Game>, send_handle: Sender<Event>) -> Box<Self> {
        Box::new(StatusTab {
            state,
            sender: send_handle,
        })
    }

    /// Returns the title string describing the tab.
    fn title(&self) -> String {
        String::from("Status")
    }

    /// Handles the user provided event.
    fn handle_event(&mut self, _event: Event) {}

    /// Draws the tab in the given terminal and area.
    fn draw(&self, term: &mut Terminal<MouseBackend>, area: &Rect) {
        Group::default()
            .direction(Direction::Horizontal)
            .sizes(&[Size::Fixed(70), Size::Min(1)])
            .render(term, area, |term, chunks| {
                let player = &self.state.player.lock().unwrap();
                draw_player_info(&player, &self.state, term, &chunks[0]);
            });
    }
}

/// Draw detailed player information.
fn draw_player_info(
    player: &Player,
    state: &Arc<Game>,
    term: &mut Terminal<MouseBackend>,
    area: &Rect,
) {
    // Data fields to be displayed in a table like format.
    let ship = player.ship();
    let galaxy = state.galaxy.lock().unwrap();
    let player_data = vec![
        format!(
            "Location:  {}",
            match player.state() {
                PlayerState::InSystem => {
                    galaxy.system(player.location()).unwrap().name.clone() + " System"
                }
                PlayerState::Docked(planet_id) => {
                    let system = galaxy.system(player.location()).unwrap();
                    let system_name = &system.name;
                    let planet_name = &system.satelites[planet_id].name;
                    format!("{}, {} System", planet_name, system_name)
                }
                _ => String::from("-"),
            }
        ),
        format!(
            "Status:    {}",
            match player.state() {
                PlayerState::Traveling { .. } => match player.eta() {
                    Some((eta, system_loc)) => match galaxy.system(&system_loc) {
                        Some(system) => {
                            format!("Traveling to {} System, ETA: {}", system.name.clone(), eta)
                        }
                        None => String::from("Bad destination"),
                    },
                    None => String::from("Stationary"),
                },
                PlayerState::InSystem => String::from("In system"),
                PlayerState::Docked(_) => String::from("Docked"),
            }
        ),
        format!("Balance:   {} CR", player.balance().to_string()),
        format!(
            "Ship:      {}",
            match *ship {
                Some(ref ship) => ship.characteristics().name.clone(),
                None => String::from("None"),
            }
        ),
        format!(
            "   Integrity: {}",
            match *ship {
                Some(ref ship) => ship.integrity().to_string(),
                None => String::from("-"),
            }
        ),
        format!(
            "   Fuel:      {}",
            match *ship {
                Some(ref ship) => ship.fuel().to_string(),
                None => String::from("-"),
            }
        ),
    ];

    // TODO: Move image to resource file.
    let commander_image = ",,,,,,,,,,,,,,,,,\",,,,,,,,,,,,,,,,,,,
,,,,,,,,,,,,,,,,,,Ii!!I:,,,,,,,,,,,,,
,,,,,,,,,,,,,,;!<+______>I:,\",,,,,,,,
,,,,,,,,,,\"\";l>_<!l;;IIl!<iI:,\",,,,,,
,,,,,,,,,,;l>++_<!I,,,,,;lI;:,,,,,,,,
,,,,,,,,,,;!<+~>!I;:,\",:,\"\"\",,,,,,,,,
,,,,,,,,,,,:Il!>iil;IllIlllll;:,,,,,,
,,,,,,,,,,,:::I!>~i;I!lIIl>+<I:,\",,,,
,,,,,,,,,,,,::::,,,,,:,,,,I>>l;,,,,,,
,,,,,,,,,,,,,,,::;:::::,,:l<>l:,\",,,,
,,,,,,,,,,,,,:::,,,,:;::Ii<+<I:,\",,,,
,,,,,,,,,,,,,:::,\",,,:::l<>l;,,,,,,,,
,,,,,,,,,,,,:;;;;::,,:::l<iI:,\",,,,,,
,,,,,,,,,,,,,,::!<i!I;;I;;:,,,,,,,,,,
,,,,,,,,,,,,,,,,:II;;;::,\"\",,,,,,,,,,
,,,,,,,,,,,,::,,,:::;;;:,,,,,,,,,,,,,
,,,,,,,,,,:::::::,,,;lII:,,,,,,,,,,,,
,,,,,,,,,:::::::,,:;:,;II;:,,,,,,,,,,
,,,,,,::::::::::::::::;I;:,,,,,,,,,,,
,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,";

    Group::default()
        .direction(Direction::Vertical)
        .sizes(&[Size::Fixed(20), Size::Min(1)])
        .render(term, area, |term, chunks| {
            Paragraph::default()
                .block(Block::default().title("Commander").borders(Borders::ALL))
                .style(Style::default().fg(Color::Yellow))
                .wrap(false)
                .text(commander_image)
                .render(term, &chunks[0]);
            SelectableList::default()
                .items(&player_data)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Yellow))
                .render(term, &chunks[1]);
        });
}
