use std::sync::Mutex;
use std::sync::Arc;
use preferences::{AppInfo, Preferences, PreferencesMap};

use astronomicals::Galaxy;

const APP_INFO: AppInfo = AppInfo {
    name: "gemini",
    author: "holmgr",
};

/// Main game state object, shared and syncronized by use of Arc and Mutex.
pub struct Game {
    pub galaxy: Mutex<Galaxy>,
}

impl Game {
    /// Creates a new game.
    pub fn new() -> Arc<Self> {
        Arc::new(Game {
            galaxy: Mutex::new(Galaxy::new(vec![])),
        })
    }

    /// Creates and stores a quicksave of the current game.
    pub fn save(&self) {
        let mut savegame = PreferencesMap::new();
        savegame.insert("galaxy".into(), (*self.galaxy.lock().unwrap()).clone());

        let prefs_key = "saves/quicksave";
        let save_result = savegame.save(&APP_INFO, prefs_key);
    }

    /// Attempts to load a quicksave of a game state.
    pub fn load() -> Option<Arc<Self>> {
        let prefs_key = "saves/quicksave";
        match PreferencesMap::load(&APP_INFO, prefs_key) {
            Ok(mut quicksave) => Some(Arc::new(Game {
                galaxy: Mutex::new(quicksave.remove("galaxy").unwrap()),
            })),
            _ => None,
        }
    }
}
