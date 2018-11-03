use chrono::{DateTime, Duration, TimeZone, Utc};
use std::time::Instant;

use core::astronomicals::Galaxy;
use core::economy::Economy;
use core::ship::Shipyard;
use player::Player;
use simulate::resources::{fetch_resource, ShipResource};

/// Main game state object, shared and syncronized by use of Arc and Mutex.
pub struct Game {
    pub galaxy: Galaxy,
    pub shipyard: Shipyard,
    pub player: Player,
    pub economy: Economy,
    pub updated: DateTime<Utc>,
}

impl Game {
    /// Creates a new game.
    pub fn new() -> Self {
        Game {
            galaxy: Galaxy::default(),
            shipyard: Shipyard::new(),
            player: Player::default(),
            economy: Economy::default(),
            updated: Utc.ymd(2018, 1, 1).and_hms(0, 0, 0), // Start time
        }
    }

    /// Attemps to advance time returning the number of days advanced if any.
    pub fn attempt_advance_time(&mut self) -> Option<i64> {
        let updated: &mut DateTime<Utc> = &mut self.updated;
        // Check if we need to advance time.
        let days_passed = Utc::now().signed_duration_since(*updated).num_days();
        if days_passed > 0 {
            // Measure time for generation.
            let now = Instant::now();
            debug!("Advancing time: {} steps", days_passed);

            // Update state iterativly.
            for _ in 0..days_passed {
                self.galaxy.update();
                self.economy.update();
            }

            // Update last update timer.
            *updated = updated
                .checked_add_signed(Duration::days(days_passed))
                .unwrap();
            //self.save_all();
            debug!(
                "Time advancement finished, took {} ms",
                ((now.elapsed().as_secs() * 1_000) + u64::from(now.elapsed().subsec_millis()))
            );
            Some(days_passed)
        } else {
            None
        }
    }
}

/// All game types which should be updated when time is advanced.
pub trait Updatable {
    /// Performs an update for one time step.
    fn update(&mut self);
}
