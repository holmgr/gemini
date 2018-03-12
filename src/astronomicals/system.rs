use std::hash::{Hash, Hasher};
use utils::Point;
use entities::Faction;
use astronomicals::hash;
use astronomicals::star::Star;
use astronomicals::planet::Planet;

#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
/// Represets a single star system with at a given location with the given
/// star and planets.
pub struct System {
    pub location: Point,
    pub name: String,
    pub faction: Faction,
    pub star: Star,
    pub satelites: Vec<Planet>,
}

impl Hash for System {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash(&self.location).hash(state);
    }
}

impl PartialEq for System {
    fn eq(&self, other: &System) -> bool {
        self.location == other.location
    }
}

impl Eq for System {}
