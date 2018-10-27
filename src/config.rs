use std::path::PathBuf;

/// Struct representing the configuration file needed when building.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub data: Data,
    pub simulation: Simulation,
}

/// Options for where the game data should be loaded from and to.
#[derive(Debug, Deserialize)]
pub struct Data {
    pub remote: String,
    pub local: PathBuf,
}

/// Parameters used for the simulation engine when generating and simulating the game.
#[derive(Debug, Deserialize)]
pub struct Simulation {
    pub map_seed: u32,
    pub starting_credits: u32,
    pub number_of_systems: u64,
    pub system_spread: f64,
    pub number_of_sectors: usize,
}
