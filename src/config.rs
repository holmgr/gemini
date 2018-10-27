use std::path::PathBuf;

/// Struct representing the configuration file needed when building.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub data: Data,
}

/// Options for where the game data should be loaded from and to.
#[derive(Debug, Deserialize)]
pub struct Data {
    remote: String,
    local: PathBuf,
}
