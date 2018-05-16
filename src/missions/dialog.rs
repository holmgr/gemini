use std::collections::HashSet;
use rand::Rng;

use resources::MissionDialogResource;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Tag {
    ReputationFriendly,
    Followup,
    End,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Line {
    required: HashSet<Tag>,
    text: String,
    results: HashSet<Tag>,
}


pub fn create_dialog<R: Rng>(dialog_resource: &MissionDialogResource, rng: &mut R, mut tags: HashSet<Tag>) -> String {
    let mut res_str = String::new();

    while !tags.contains(&Tag::End) {

        let mut line_options = vec![];
        let mut matches_count = 0;

        for line in dialog_resource.dialog_options.iter() {
            if line.required.is_subset(&tags) {
                if line.required.len() > matches_count {
                    matches_count = line.required.len();
                    line_options = vec![line.clone()];
                }
                else if line.required.len() == matches_count {
                    line_options.push(line.clone());
                }
            }
        }

        let next_line = rng.choose(&line_options).unwrap().clone();

        res_str.push_str(&next_line.text);
        res_str.push('\n');
        tags.extend(next_line.results);
    }

    res_str
}

#[cfg(test)]
mod tests {
    use rand::{ChaChaRng, SeedableRng};

    use super::*;

    use setup_logger;
    use resources::fetch_resource;

    #[test]
    fn test_dialog() {
        // Init logger
        setup_logger();

        let resource = fetch_resource::<MissionDialogResource>().unwrap();
        let tags = HashSet::new();
        let mut rng = ChaChaRng::from_seed(&[42]);

        for _ in 0..10 {
            println!("{}", create_dialog(&resource, &mut rng, tags.clone()));
        }
    }
}
