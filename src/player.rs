use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use nalgebra::distance;

use utils::Point;
use ship::Ship;

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Player type holding the player's current ship, credits etc.
pub struct Player {
    credits: u32,
    ship: Option<Ship>,
    location: Point,
    state: PlayerState,
}

impl Player {
    /// Travling speed between systems, ly/ms.
    const TRAVEL_SPEED: f64 = 10. / 60000.;

    /// Create a new player.
    pub fn new(credits: u32, ship: Ship, location: &Point) -> Self {
        Player {
            credits,
            ship: Some(ship),
            location: location.clone(),
            state: PlayerState::InSystem,
        }
    }

    /// Update the player state.
    pub fn update_state(&mut self) {
        // Should we continue to update?
        let mut repeat = true;
        while repeat {
            repeat = false;
            self.state = match self.state {
                PlayerState::Docked(planetid) => PlayerState::Docked(planetid),
                PlayerState::InSystem => PlayerState::InSystem,
                PlayerState::Traveling {
                    ref start,
                    ref route,
                } => {
                    match route.split_first() {
                        // Arrived at next system in route?
                        Some((next, rest))
                            if distance(&self.location, &route[0])
                                <= Utc::now().signed_duration_since(*start).num_milliseconds()
                                    as f64
                                    * Player::TRAVEL_SPEED =>
                        {
                            let new_start = *start
                                + Duration::milliseconds(
                                    (&distance(&self.location, &next) / Player::TRAVEL_SPEED)
                                        as i64,
                                );

                            // Update position and reduce fuel.
                            self.location = *next;
                            if let Some(ref mut ship) = self.ship {
                                ship.reduce_fuel();
                            }

                            // Maybe we can move one step more already.
                            repeat = true;
                            PlayerState::Traveling {
                                start: new_start,
                                route: rest.to_vec(),
                            }
                        }
                        // Not yet arrived?
                        Some((next, rest)) => {
                            let mut combined = vec![next];
                            combined.extend(rest);
                            PlayerState::Traveling {
                                start: *start,
                                route: route.to_vec(),
                            }
                        }
                        // No route left.
                        None => PlayerState::InSystem,
                    }
                }
            };
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

    /// Returns the player state.
    pub fn state(&self) -> PlayerState {
        self.state.clone()
    }

    /// Docks the player to the planet with the given id.
    pub fn dock(&mut self, planet_id: usize) {
        self.state = PlayerState::Docked(planet_id);
    }

    /// Undocks the player from its current planet.
    pub fn undock(&mut self) {
        self.state = PlayerState::InSystem;
    }

    /// Attemps to fuel up the player ship as far as credits reaches.
    pub fn refuel(&mut self) {
        if let Some(ref mut ship) = self.ship {
            // TODO: Assumes each fuel unit costs 10 credits.
            let to_fill = (ship.characteristics().fuel - ship.fuel()).min(self.credits / 10);
            self.credits -= to_fill * 10;
            ship.add_fuel(to_fill);
        }
    }

    /// Sets the route for the player.
    pub fn set_route(&mut self, route: Vec<Point>) {
        self.state = PlayerState::Traveling {
            start: Utc::now(),
            route: route,
        };
    }

    /// Get the player's currrent route, if available.
    pub fn route(&self) -> Option<Vec<&Point>> {
        match self.state {
            PlayerState::Traveling { ref route, .. } => Some(route.iter().collect::<Vec<_>>()),
            _ => None,
        }
    }

    /// Get the estimated time of arrival (local time) and the destination of the current route.
    pub fn eta(&self) -> Option<(String, Point)> {
        match self.state {
            PlayerState::Traveling {
                ref start,
                ref route,
            } => {
                let (dist, destination) = route
                    .iter()
                    .fold((0., &self.location), |(dist, prev), curr| {
                        (dist + distance(prev, curr), curr)
                    });
                let eta = Local.from_utc_datetime(
                    &(*start + Duration::milliseconds((dist / Player::TRAVEL_SPEED) as i64))
                        .naive_utc(),
                );
                // Format in HH:MM:SS and AM/PM.
                Some((eta.format("%r").to_string(), destination.clone()))
            }
            _ => None,
        }
    }
}

impl Default for Player {
    fn default() -> Player {
        Player {
            credits: 0,
            ship: None,
            location: Point::origin(),
            state: PlayerState::InSystem,
        }
    }
}

/// Holds the current state of the player which affects the options of interaction.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PlayerState {
    InSystem,
    Docked(usize),
    Traveling {
        start: DateTime<Utc>,
        route: Vec<Point>,
    },
}
