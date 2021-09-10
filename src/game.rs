use chrono::{DateTime, Duration, TimeZone, Utc};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use astronomicals::Galaxy;
use economy::Economy;

/// Main game state object, shared and syncronized by use of Arc and Mutex.
pub struct Game {
    pub galaxy: Mutex<Galaxy>,
    pub economy: Mutex<Economy>,
    updated: Mutex<DateTime<Utc>>,
}

impl Game {
    /// Creates a new game.
    pub fn new() -> Arc<Self> {
        Arc::new(Game {
            galaxy: Mutex::new(Galaxy::default()),
            economy: Mutex::new(Economy::default()),
            updated: Mutex::new(Utc.ymd(2021, 1, 1).and_hms(0, 0, 0)), // Start time
        })
    }

    /// Update Game information, may advance time.
    pub fn update(&self) {
        // If we have advanced time some steps.
        self.attempt_advance_time();
    }

    /// Attemps to advance time returning the number of days advanced if any.
    fn attempt_advance_time(&self) -> Option<i64> {
        let updated: &mut DateTime<Utc> = &mut self.updated.lock().unwrap();
        // Check if we need to advance time.
        let days_passed = Utc::now().signed_duration_since(*updated).num_days();
        if days_passed > 0 {
            // Measure time for generation.
            let now = Instant::now();
            debug!("Advancing time: {} steps", days_passed);

            // Update state iterativly.
            for _ in 0..days_passed {
                self.galaxy.lock().unwrap().update();
                self.economy.lock().unwrap().update();
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
