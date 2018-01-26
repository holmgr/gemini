use rand::Rng;

/// Represents a single Faction which is assigned on Sector level.
#[derive(Debug)]
pub enum Faction {
    Empire,
    Federation,
    Cartel,
    Independent,
}

impl Faction {
    /// Generate a random faction according to the "distribution".
    pub fn random_faction<R: Rng>(mut gen: R) -> Faction {
        let probs = vec![
            (Faction::Cartel, 12),
            (Faction::Empire, 50),
            (Faction::Federation, 30),
            (Faction::Independent, 8),
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
