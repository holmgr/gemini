use utils::Point;
use ship::Ship;

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Player type holding the player's current ship, credits etc.
pub struct Player {
    credits: u32,
    ship: Option<Ship>,
    location: Point,
}

impl Player {
    /// Create a new player.
    pub fn new(credits: u32, ship: Ship, location: &Point) -> Self {
        Player {
            credits,
            ship: Some(ship),
            location: location.clone(),
        }
    }

    /// Returns the player's current balance.
    pub fn balance(&self) -> u32 {
        self.credits
    }

    /// Returns an reference to the player's active ship.
    pub fn ship(&self) -> &Option<Ship> {
        &self.ship
    }

    /// Returns an reference to the player's active ship.
    pub fn ship_mut(&mut self) -> &mut Option<Ship> {
        &mut self.ship
    }

    /// Get the current player location.
    pub fn location(&self) -> &Point {
        &self.location
    }

    /// Sets the player location.
    pub fn set_location(&mut self, location: &Point) {
        self.location = location.clone();
    }
}

impl Default for Player {
    fn default() -> Player {
        Player {
            credits: 0,
            ship: None,
            location: Point::origin(),
        }
    }
}
