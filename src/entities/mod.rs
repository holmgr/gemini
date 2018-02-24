use rand::Rng;
use std::fmt;

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
        let probs = vec![
            (Faction::Cartel, 15),
            (Faction::Empire, 45),
            (Faction::Federation, 30),
            (Faction::Independent, 10),
        ];

        let mut rnd: u32 = gen.gen_range(0, 100);
        for (fac, prob) in probs {
            if rnd <= prob {
                return fac;
            } else {
                rnd -= prob;
            }
        }
        // Default faction
        Faction::Independent
    }
}
impl fmt::Display for Faction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
