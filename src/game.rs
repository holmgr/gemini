use app_dirs::{get_data_root, AppDataType};
use bincode::{deserialize_from, serialize_into};
use chrono::{DateTime, Duration, TimeZone, Utc};
use std::{
    fs::{create_dir_all, File}, io::{BufReader, BufWriter}, sync::{Arc, Mutex}, time::Instant,
};

use astronomicals::Galaxy;
use economy::Economy;
use player::Player;
use resources::{fetch_resource, ShipResource};
use ship::Shipyard;

const SAVE_PATH: &str = "gemini/saves/";

/// Main game state object, shared and syncronized by use of Arc and Mutex.
pub struct Game {
    pub galaxy: Mutex<Galaxy>,
    pub shipyard: Mutex<Shipyard>,
    pub player: Mutex<Player>,
    pub economy: Mutex<Economy>,
    updated: Mutex<DateTime<Utc>>,
}

impl Game {
    /// Creates a new game.
    pub fn new() -> Arc<Self> {
        Arc::new(Game {
            galaxy: Mutex::new(Galaxy::default()),
            shipyard: Mutex::new(Shipyard::new()),
            player: Mutex::new(Player::default()),
            economy: Mutex::new(Economy::default()),
            updated: Mutex::new(Utc.ymd(2018, 1, 1).and_hms(0, 0, 0)), // Start time
        })
    }

    /// Update Game information, may advance time.
    pub fn update(&self) {
        // If we have advanced time some steps.
        if self.attempt_advance_time().is_some() {
            self.save_all();
        }

        // Update player location etc.
        self.player.lock().unwrap().update_state();
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
            serialize_into(&mut galaxy_file, &(*self.galaxy.lock().unwrap()));
            let mut player_file =
                BufWriter::new(File::create(base_path.join("player.cbor").as_path()).unwrap());
            serialize_into(&mut player_file, &(*self.player.lock().unwrap()));
            let mut economy_file =
                BufWriter::new(File::create(base_path.join("economy.cbor").as_path()).unwrap());
            serialize_into(&mut economy_file, &(*self.economy.lock().unwrap()));
            let mut update_file =
                BufWriter::new(File::create(base_path.join("updated.cbor").as_path()).unwrap());
            serialize_into(&mut update_file, &(*self.updated.lock().unwrap()));
        }
    }

    /// Creates and stores a quicksave of the player data.
    pub fn save_player(&self) {
        let base_path = get_data_root(AppDataType::UserConfig)
            .unwrap()
            .join(SAVE_PATH);

        if create_dir_all(base_path.as_path()).is_ok() {
            let mut player_file =
                BufWriter::new(File::create(base_path.join("player.cbor").as_path()).unwrap());
            serialize_into(&mut player_file, &(*self.player.lock().unwrap()));
        }
    }

    /// Attempts to load a quicksave of a game state.
    pub fn load() -> Option<Arc<Self>> {
        let base_path = get_data_root(AppDataType::UserConfig)
            .unwrap()
            .join(SAVE_PATH);

        let galaxy: Option<Galaxy> = File::open(base_path.join("galaxy.cbor").as_path())
            .ok()
            .and_then(|galaxy_file| deserialize_from(BufReader::new(galaxy_file)).ok());

        let player: Option<Player> = File::open(base_path.join("player.cbor").as_path())
            .ok()
            .and_then(|player_file| deserialize_from(BufReader::new(player_file)).ok());
        let economy: Option<Economy> = File::open(base_path.join("economy.cbor").as_path())
            .ok()
            .and_then(|economy_file| deserialize_from(BufReader::new(economy_file)).ok());
        let updated: Option<DateTime<Utc>> = File::open(base_path.join("updated.cbor").as_path())
            .ok()
            .and_then(|updated_file| deserialize_from(BufReader::new(updated_file)).ok());

        let mut shipyard = Shipyard::new();
        shipyard.add_ships(fetch_resource::<ShipResource>().unwrap());

        match (galaxy, player, economy, updated) {
            (Some(g), Some(p), Some(e), Some(u)) => Some(Arc::new(Game {
                galaxy: Mutex::new(g),
                shipyard: Mutex::new(shipyard),
                player: Mutex::new(p),
                economy: Mutex::new(e),
                updated: Mutex::new(u),
            })),
            _ => None,
        }
    }
}

/// All game types which should be updated when time is advanced.
pub trait Updatable {
    /// Performs an update for one time step.
    fn update(&mut self);
}
