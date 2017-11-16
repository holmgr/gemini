use preferences::{AppInfo, Preferences, PreferencesError, prefs_base_dir};
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::{channel, RecvError};
use std::time::Duration;

// Configuration level constants, location for configs is determined by this
const APP_INFO: AppInfo = AppInfo {
    name: "gemini",
    author: "holmgr",
};
const PREFS_KEY: &str = "conf/general";
const DEFAULT_SEED: u32 = 42;

// Deriving `Serialize` and `Deserialize` on a struct/enum automatically
// implements the `Preferences` trait.
/// Contains high level configuration parameters for the game such as constants
/// for generation.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct GameConfig {
    map_seed: u32,
}

impl GameConfig {
    /// Attempts to load a GameConfig from disk at the default preference
    /// location.
    /// If the loading fails for any reason, for example, the file does not
    /// exist, a new default GameConfig object is created, stored and returned.
    pub fn retrieve() -> GameConfig {
        match GameConfig::load(&APP_INFO, PREFS_KEY) {
            Ok(config) => config,
            _ => {
                info!("Failed loading config, serving default");
                let config = GameConfig { map_seed: DEFAULT_SEED };
                let _ = config.store();
                config
            }
        }
    }

    /// Attempts to store the GameConfig on disk at the default preference
    /// location.
    /// # Failures
    /// If a serialization or file I/O error (e.g. permission denied) occurs.
    pub fn store(&self) -> Result<(), PreferencesError> {
        self.save(&APP_INFO, PREFS_KEY)
    }

    /// Setup a blocking Watcher listening for any file changes at the
    /// preferences location of the GameConfig.
    /// If some change is detected it will attempt to return the updated
    /// GameConfig.
    /// # Failures
    /// Channel error when attempting to read while watching for changes.
    pub fn await_update() -> Result<GameConfig, RecvError> {
        // Create a channel to receive the events.
        let (tx, rx) = channel();

        // Create a watcher object, delivering debounced events.
        // The notification back-end is selected based on the platform.
        let mut watcher = watcher(tx, Duration::from_secs(30)).unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        let mut config_path = prefs_base_dir().unwrap();
        config_path.push(APP_INFO.name);
        config_path.push(format!("{}.prefs.json", PREFS_KEY));
        watcher
            .watch(config_path, RecursiveMode::Recursive)
            .unwrap();

        match rx.recv() {
            Ok(_) => Ok(GameConfig::retrieve()),
            Err(e) => Err(e),
        }
    }
}
