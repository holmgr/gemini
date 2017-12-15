use std::str;
use serde_json;
use serde::de::Deserialize;
use std::collections::HashMap;

/// Generic Resource trait to be implemented by all Resource types which should
/// be loaded at compile time
/// KEY must be unique to the specific resouce (e.g the filename of the
/// resource)
pub trait Resource: Deserialize {
    const KEY: &'static str;
}

lazy_static! {
    // Load resources at compile time
    // TODO: Convert to resource at compile time to save resources
    static ref RESOURCES: HashMap<&'static str, &'static str> = {
        let mut res = HashMap::new();
        res.insert(
            AstronomicalNamesResource::KEY,
            include_str!("../res/astronomical_names.json"),
        );
        res.insert(
            StarTypesResource::KEY,
            include_str!("../res/star_types.json"),
        );
        res.insert(
            PlanetTypesResource::KEY,
            include_str!("../res/planet_types.json"),
        );
        res
    };
}

/// Attempts to returns the resource with the given type, will return None
/// if the type has no resource or if the deserialization fails
pub fn fetch_resource<T: Resource>() -> Option<T> {
    RESOURCES.get(T::KEY).and_then(|res: &&str| {
        serde_json::from_str(res).unwrap_or(None)
    })
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
/// Resource of all training names for name generation of celestial objects
pub struct AstronomicalNamesResource {
    pub names: Vec<String>,
    pub greek: Vec<String>,
    pub decorators: Vec<String>,
}

impl Resource for AstronomicalNamesResource {
    const KEY: &'static str = "astronomical_names";
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
/// Resource of different star types of stellar objects
pub struct StarTypesResource {
    pub main_sequence: Vec<StarType>,
    pub giant: Vec<StarType>,
    pub supergiant: Vec<StarType>,
}

/// Struct describing one type of star
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct StarType {
    pub kind: String,
    pub surface_temp: u32,
    pub radius: f64,
    pub mass: f64,
    pub absolute_magnitude: f64,
    pub luminosity: f64,
    pub habitable_zone: f64,
    pub abundance: f64,
}

impl Resource for StarTypesResource {
    const KEY: &'static str = "star_types";
}

/// Struct describing one type of star
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlanetTypesResource {}

impl Resource for PlanetTypesResource {
    const KEY: &'static str = "planet_types";
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate env_logger;

    #[test]
    fn test_fetch_resource() {
        // Init logger
        let _ = env_logger::init();

        let res = fetch_resource::<AstronomicalNamesResource>().unwrap();
        assert!(res.names.len() > 0);
        assert!(res.greek.len() > 0);
        assert!(res.decorators.len() > 0);
    }
}
