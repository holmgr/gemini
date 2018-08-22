use super::*;

/// Economic agent connected to a system, takes part in economy buy trading/producing.
#[derive(Serialize, Deserialize, Debug)]
pub struct Agent {}

impl Agent {
    /// Create a new economic agent for the given system.
    pub fn new(system: &System, population: u64) -> Self {
        Agent {}
    }
}
