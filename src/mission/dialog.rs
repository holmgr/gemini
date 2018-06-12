use rand::Rng;
use std::collections::HashSet;

use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tag(String);

impl Tag {
    pub fn end() -> Self {
        Tag {
            0: String::from("End"),
        }
    }

    pub fn as_tag<T: fmt::Display>(value: T) -> Self {
        Tag {
            0: value.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Line {
    required: HashSet<Tag>,
    text: String,
    results: HashSet<Tag>,
}

pub fn create_dialog<R: Rng>(
    dialog_resource: &MissionDialogResource,
    rng: &mut R,
    mut tags: HashSet<Tag>,
) -> String {
    let mut res_str = String::new();

    while !tags.contains(&Tag::end()) {
        let mut line_options = vec![];
        let mut matches_count = 0;

        for line in &dialog_resource.dialog_options {
            if line.required.is_subset(&tags) {
                if line.required.len() > matches_count {
                    matches_count = line.required.len();
                    line_options = vec![line.clone()];
                } else if line.required.len() == matches_count {
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

    use resources::fetch_resource;
    use setup_logger;

    #[test]
    fn test_dialog() {
        // Init logger
        setup_logger();

        let resource = fetch_resource::<MissionDialogResource>().unwrap();
        let mut rng = ChaChaRng::from_seed(&[42]);

        for _ in 0..10 {
            //let tags = HashSet::new();
            //println!("{}", create_dialog(&resource, &mut rng, tags));
        }
    }
}
