use std::sync::Mutex;
use std::sync::Arc;
use std::fs::{create_dir_all, File};
use preferences::prefs_base_dir;
use serde_cbor::{from_reader, to_writer};

use astronomicals::Galaxy;
use ship::Shipyard;
use player::Player;
use resources::{fetch_resource, ShipResource};

const SAVE_PATH: &str = "gemini/saves/";

/// Main game state object, shared and syncronized by use of Arc and Mutex.
pub struct Game {
    pub galaxy: Mutex<Galaxy>,
    pub shipyard: Mutex<Shipyard>,
    pub player: Mutex<Player>,
}

impl Game {
    /// Creates a new game.
    pub fn new() -> Arc<Self> {
        Arc::new(Game {
            galaxy: Mutex::new(Galaxy::new(vec![], vec![])),
            shipyard: Mutex::new(Shipyard::new()),
            player: Mutex::new(Player::default()),
        })
    }

    /// Creates and stores a quicksave of the current game.
    pub fn save_all(&self) {
        let base_path = prefs_base_dir().unwrap().join(SAVE_PATH);

        create_dir_all(base_path.as_path())
            .ok()
            .and_then(|_| File::create(base_path.join("galaxy.cbor").as_path()).ok())
            .and_then(|mut galaxy_file|
                // Save galaxy
                to_writer(&mut galaxy_file, &(*self.galaxy.lock().unwrap())).ok())
            .and_then(|_| File::create(base_path.join("player.cbor").as_path()).ok())
            .and_then(|mut player_file|
                // Save galaxy
                to_writer(&mut player_file, &(*self.player.lock().unwrap())).ok());
    }

    /// Creates and stores a quicksave of the player data.
    pub fn save_player(&self) {
        let base_path = prefs_base_dir().unwrap().join(SAVE_PATH);

        create_dir_all(base_path.as_path())
            .ok()
            .and_then(|_| File::create(base_path.join("player.cbor").as_path()).ok())
            .and_then(|mut player_file|
                // Save galaxy
                to_writer(&mut player_file, &(*self.player.lock().unwrap())).ok());
    }

    /// Attempts to load a quicksave of a game state.
    pub fn load() -> Option<Arc<Self>> {
        let base_path = prefs_base_dir().unwrap().join(SAVE_PATH);

        let galaxy: Option<Galaxy> = File::open(base_path.join("galaxy.cbor").as_path())
            .ok()
            .and_then(|galaxy_file| from_reader(galaxy_file).ok());

        let player: Option<Player> = File::open(base_path.join("player.cbor").as_path())
            .ok()
            .and_then(|player_file| from_reader(player_file).ok());

        let mut shipyard = Shipyard::new();
        shipyard.add_ships(fetch_resource::<ShipResource>().unwrap());

        match (galaxy, player) {
            (Some(g), Some(p)) => Some(Arc::new(Game {
                galaxy: Mutex::new(g),
                shipyard: Mutex::new(shipyard),
                player: Mutex::new(p),
            })),
            _ => None,
        }
    }
}
