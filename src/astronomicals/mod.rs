
use generators::stars::StarGen;
use generators::names::NameGen;
use generators::planets::PlanetGen;
use generators::MutGen;
use generators::Gen;

pub mod star;
pub mod planet;
pub mod system;

#[derive(Debug)]
pub struct Galaxy {
    systems: Vec<system::System>,
}

impl Galaxy {
    pub fn new(systems: Vec<system::System>) -> Self {
        Galaxy { systems }
    }
}
