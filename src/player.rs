use ship::Ship;

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Player type holding the player's current ship, credits etc.
pub struct Player {
    credits: u32,
    ship: Option<Ship>,
}

impl Player {
    /// Create a new player.
    pub fn new(credits: u32, ship: Ship) -> Self {
        Player {
            credits,
            ship: Some(ship),
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
}

impl Default for Player {
    fn default() -> Player {
        Player {
            credits: 0,
            ship: None,
        }
    }
}
