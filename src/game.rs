use app_dirs::{get_data_root, AppDataType};
use bincode::serialize_into;
use chrono::{DateTime, Duration, TimeZone, Utc};
use std::{
    fs::{create_dir_all, File},
    io::BufWriter,
    sync::{Arc, Mutex},
    time::Instant,
};

use astronomicals::Galaxy;
use economy::Economy;

const SAVE_PATH: &str = "gemini/saves/";

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
        if self.attempt_advance_time().is_some() {
            self.save_all();
        }
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

    /// Creates and stores a quicksave of the current game.
    pub fn save_all(&self) {
        let base_path = get_data_root(AppDataType::UserConfig)
            .unwrap()
            .join(SAVE_PATH);

        if create_dir_all(base_path.as_path()).is_ok() {
            let mut galaxy_file =
                BufWriter::new(File::create(base_path.join("galaxy.cbor").as_path()).unwrap());
            serialize_into(&mut galaxy_file, &(*self.galaxy.lock().unwrap())).unwrap();
            let mut economy_file =
                BufWriter::new(File::create(base_path.join("economy.cbor").as_path()).unwrap());
            serialize_into(&mut economy_file, &(*self.economy.lock().unwrap())).unwrap();
            let mut update_file =
                BufWriter::new(File::create(base_path.join("updated.cbor").as_path()).unwrap());
            serialize_into(&mut update_file, &(*self.updated.lock().unwrap())).unwrap();
        }
    }
}

/// All game types which should be updated when time is advanced.
pub trait Updatable {
    /// Performs an update for one time step.
    fn update(&mut self);
}
