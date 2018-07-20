use app_dirs::{get_data_root, AppDataType};
use notify::{watcher, RecursiveMode, Watcher};
use std::{
    fs::{create_dir_all, File}, io::{Read, Write}, sync::mpsc::{channel, RecvError}, time::Duration,
};
use toml::{de::from_str, ser::to_string_pretty};

const PREFS_PATH: &str = "gemini/conf/";

// Deriving `Serialize` and `Deserialize` on a struct/enum automatically
// implements the `Preferences` trait.
/// Contains high level configuration parameters for the game such as constants
/// for generation.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct GameConfig {
    pub map_seed: u32,
    pub starting_credits: u32,
    pub number_of_systems: u64,
    pub system_spread: f64,
    pub number_of_sectors: usize,
}

impl GameConfig {
    /// Attempts to load a GameConfig from disk at the default preference
    /// location.
    /// If the loading fails for any reason, for example, the file does not
    /// exist, a new default GameConfig object is created, stored and returned.
    pub fn retrieve() -> GameConfig {
        let config: Option<GameConfig> = File::open(
            get_data_root(AppDataType::UserConfig)
                .unwrap()
                .join(PREFS_PATH)
                .join("general.toml")
                .as_path(),
        ).ok()
            .and_then(|mut config_file| {
                let mut config_str = String::new();
                match config_file.read_to_string(&mut config_str) {
                    Ok(_) => Some(config_str),
                    Err(_) => None,
                }
            })
            .and_then(|config_str| from_str(&config_str).ok())
            .or_else(|| None);

        match config {
            Some(config) => config,
            None => {
                info!("Failed loading config, serving default");
                let config: GameConfig = Default::default();
                config.store();
                config
            }
        }
    }

    /// Attempts to store the GameConfig on disk at the default preference
    /// location.
    pub fn store(&self) {
        let base_path = get_data_root(AppDataType::UserConfig)
            .unwrap()
            .join(PREFS_PATH);

        create_dir_all(base_path.as_path())
            .ok()
            .and_then(|_| File::create(base_path.join("general.toml").as_path()).ok())
            .and_then(|mut config_file| {
                let encoded = to_string_pretty(self).unwrap();
                config_file.write_all(&encoded.into_bytes()).ok()
            });
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
        let config_path = get_data_root(AppDataType::UserConfig)
            .unwrap()
            .join(PREFS_PATH);
        watcher
            .watch(config_path, RecursiveMode::Recursive)
            .unwrap();

        match rx.recv() {
            Ok(_) => Ok(GameConfig::retrieve()),
            Err(e) => Err(e),
        }
    }
}

impl Default for GameConfig {
    fn default() -> GameConfig {
        GameConfig {
            map_seed: 42,
            starting_credits: 1000,
            number_of_systems: 10_000,
            system_spread: 150.,
            number_of_sectors: 150,
        }
    }
}
