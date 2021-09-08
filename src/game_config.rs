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
