use rand::Rng;
use statrs::distribution::{Categorical, Distribution};
use std::{collections::HashMap, sync::Arc};

use game::Game;

pub mod dialog;

type Context = HashMap<String, String>;

/// Number of missions to be generated in one set.
const MISSION_COUNT: usize = 5;

/// Generates a set of missions for the current location based on world state.
pub fn gen_missions<R: Rng>(gen: &mut R, state: Arc<Game>) -> Vec<Mission> {
    // TODO: Generate context object from game state
    let context = Context::new();

    let mut missions = vec![];
    for _ in 0..MISSION_COUNT {
        missions.push(Mission::gen(&context, gen));
    }
    missions
}

/// A user achivable mission.
#[derive(Debug)]
pub struct Mission {
    motivation: Motivation,
    description: String,
    actions: Vec<Action>,
}

impl Mission {
    /// Randomly create a new mission.
    pub fn gen<R: Rng>(context: &Context, gen: &mut R) -> Self {
        let motivation = Motivation::gen(gen);
        let (description, actions) = motivation.gen_strategy(gen);

        Mission {
            motivation,
            description,
            actions,
        }
    }
}

/// NPC motivation behind a given mission.
#[derive(Debug)]
enum Motivation {
    Knowledge,
    Protection,
    Reputation,
    Wealth,
}

impl Motivation {
    /// Randomly create a new motivation.
    pub fn gen<R: Rng>(gen: &mut R) -> Self {
        let variants = Categorical::new(&[20., 20., 20., 20.]).unwrap();
        match variants.sample(gen) as usize {
            0 => Motivation::Knowledge,
            1 => Motivation::Protection,
            2 => Motivation::Reputation,
            _ => Motivation::Wealth,
        }
    }

    /// Generate a random strategy.
    pub fn gen_strategy<R: Rng>(&self, gen: &mut R) -> (String, Vec<Action>) {
        let choices = match self {
            Motivation::Knowledge => vec![
                (
                    String::from("Deliver item for study"),
                    vec![Action::Get, Action::Goto, Action::Give],
                ),
                (
                    String::from("Interview NPC"),
                    vec![Action::Goto, Action::Listen, Action::Goto, Action::Report],
                ),
                (
                    String::from("Use item in the field"),
                    vec![
                        Action::Get,
                        Action::Goto,
                        Action::Use,
                        Action::Goto,
                        Action::Give,
                    ],
                ),
            ],
            Motivation::Protection => vec![
                (
                    String::from("Check on NPC"),
                    vec![Action::Goto, Action::Listen, Action::Goto, Action::Report],
                ),
                (
                    String::from("Smuggle out NPC"),
                    vec![Action::Goto, Action::Smuggle, Action::Goto, Action::Report],
                ),
            ],
            Motivation::Reputation => vec![(String::from("Donate items"), vec![Action::Donate])],
            Motivation::Wealth => vec![
                (
                    String::from("Deliver supplies"),
                    vec![Action::Get, Action::Goto, Action::Give],
                ),
            ],
        };

        gen.choose(&choices).unwrap().clone()
    }
}

/// Actions that makes up a mission.
#[derive(Clone, Debug)]
enum Action {
    Buy,
    Donate,
    Get,
    Give,
    Goto,
    Listen,
    Report,
    Smuggle,
    Use,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{ChaChaRng, SeedableRng};
    use setup_logger;

    #[test]
    fn test_mission_gen() {
        setup_logger();

        let state = Game::new();

        let mut rng = ChaChaRng::from_seed(&[42]);
        println!("{:#?}", gen_missions(&mut rng, state));
    }
}
