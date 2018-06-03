use rand::Rng;
use statrs::distribution::{Categorical, Distribution};
use std::{collections::{HashSet, HashMap}, sync::Arc, fmt};

use resources::{fetch_resource, MissionDialogResource};
use game::Game;

pub mod dialog;
use self::dialog::{Tag, create_dialog};

type Context = HashMap<String, String>;

/// Number of missions to be generated in one set.
const MISSION_COUNT: usize = 5;

/// Generates a set of missions for the current location based on world state.
pub fn gen_missions<R: Rng>(gen: &mut R, state: &Arc<Game>) -> Vec<Mission> {
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
        let resource = fetch_resource::<MissionDialogResource>().unwrap();

        let motivation = Motivation::gen(gen);
        let actions = motivation.gen_strategy(gen);

        let mut tags = HashSet::<Tag>::new();
        tags.insert(Tag::as_tag(motivation));

        let description = create_dialog(&resource, gen, tags);
        Mission {
            motivation,
            description,
            actions,
        }
    }
}

/// NPC motivation behind a given mission.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
    pub fn gen_strategy<R: Rng>(&self, gen: &mut R) -> Vec<Action> {
        let choices = match self {
            Motivation::Knowledge => vec![
                // Deliver item for study
                vec![Action::Get, Action::Goto, Action::Give],
                // Interview NPC
                vec![Action::Goto, Action::Listen, Action::Goto, Action::Report],
                // Use item in the field
                vec![
                    Action::Get,
                    Action::Goto,
                    Action::Use,
                    Action::Goto,
                    Action::Give,
                ],
            ],
            Motivation::Protection => vec![
                // Check on NPC
                vec![Action::Goto, Action::Listen, Action::Goto, Action::Report],
                // Smuggle out NPC
                vec![Action::Goto, Action::Smuggle, Action::Goto, Action::Report],
            ],
            Motivation::Reputation => vec![
                // Donate items
                vec![Action::Donate]
            ],
            Motivation::Wealth => vec![
                // Deliver supplies
                vec![Action::Get, Action::Goto, Action::Give],
           ],
        };

        gen.choose(&choices).unwrap().clone()
    }
}

impl fmt::Display for Motivation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
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
    use serde_json;
    use setup_logger;

    #[test]
    fn test_mission_gen() {
        setup_logger();

        let state = Game::new();

        let tag = Tag::as_tag(Motivation::Wealth);
        let j = serde_json::to_string(&tag).unwrap();
        println!("{}", j);

        let mut rng = ChaChaRng::from_seed(&[42]);
        println!("{:#?}", gen_missions(&mut rng, &state));
    }
}
