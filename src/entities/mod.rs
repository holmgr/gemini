use rand::Rng;
use std::fmt;
use statrs::distribution::{Categorical, Distribution};

/// Represents a single Faction which is assigned on Sector level.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Faction {
    Empire,
    Federation,
    Cartel,
    Independent,
}

impl Faction {
    /// Generate a random faction according to the "distribution".
    pub fn random_faction<R: Rng>(gen: &mut R) -> Faction {
        let probs = Categorical::new(&[15., 45., 30., 10.]).unwrap();

        match probs.sample::<R>(gen) as usize {
            0 => Faction::Cartel,
            1 => Faction::Empire,
            2 => Faction::Federation,
            3 => Faction::Independent,
            _ => Faction::Independent,
        }
    }
}
impl fmt::Display for Faction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
