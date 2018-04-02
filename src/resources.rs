use std::{str, collections::HashMap};
use serde_json;
use serde::de::Deserialize;

use ship::ShipCharacteristics;

/// Generic Resource trait to be implemented by all resource types which should
/// be loaded at compile time.
/// KEY must be unique to the specific resource (e.g the filename of the
/// resource).
pub trait Resource: Deserialize<'static> {
    const KEY: &'static str;
}

lazy_static! {
    // Load resources at compile time.
    // TODO: Convert to resource at compile time to save resources.
    static ref RESOURCES: HashMap<&'static str, &'static str> = {
        let mut res = HashMap::new();
        res.insert(
            AstronomicalNamesResource::KEY,
            include_str!("../res/astronomical_names.json"),
        );
        res.insert(
            ShipResource::KEY,
            include_str!("../res/ships.json"),
        );
        res
    };
}

/// Attempts to returns the resource with the given type, will return None
/// if the type has no resource or if the deserialization fails.
pub fn fetch_resource<T: Resource>() -> Option<T> {
    RESOURCES
        .get(T::KEY)
        .and_then(|res: &&str| serde_json::from_str(res).unwrap_or(None))
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
/// Resource of all training names for name generation of celestial objects.
pub struct AstronomicalNamesResource {
    pub names: Vec<String>,
    pub greek: Vec<String>,
    pub decorators: Vec<String>,
}

impl Resource for AstronomicalNamesResource {
    const KEY: &'static str = "astronomical_names";
}

#[derive(Serialize, Deserialize, Debug)]
/// Resource of all ships available in the game.
pub struct ShipResource {
    pub ships: Vec<ShipCharacteristics>,
}

impl Resource for ShipResource {
    const KEY: &'static str = "ships";
}

#[cfg(test)]
mod tests {
    use super::*;
    use setup_logger;

    #[test]
    fn test_fetch_resource() {
        // Init logger
        setup_logger();

        let res = fetch_resource::<AstronomicalNamesResource>().unwrap();
        assert!(res.names.len() > 0);
        assert!(res.greek.len() > 0);
        assert!(res.decorators.len() > 0);
    }
}
