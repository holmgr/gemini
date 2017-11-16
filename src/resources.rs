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

/// Resource factory which holds all resources serialized based on the Resource
/// KEY.
pub struct ResourceHandler {
    resources: HashMap<&'static str, &'static str>,
}

impl ResourceHandler {
    /// Creates a new ResourceHandler, initilaizing the map of resources
    pub fn new() -> ResourceHandler {

        // Init map of resources to be included
        let mut resources = HashMap::new();
        resources.insert(
            AstronomicalNamesResource::KEY,
            include_str!("../res/astronomical_names.json"),
        );

        ResourceHandler { resources }
    }

    /// Attempts to returns the resource with the given type, will return None
    /// if the type has no resource or if the deserialization fails
    pub fn fetch_resource<T: Resource>(&self) -> Option<T> {
        self.resources.get(T::KEY).and_then(|res: &&str| {
            serde_json::from_str(res).unwrap_or(None)
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
/// Resource of all training names for name generation of celestial objects
pub struct AstronomicalNamesResource {
    pub names: Vec<String>,
    pub greek: Vec<String>,
    pub decorators: Vec<String>,
}

impl Resource for AstronomicalNamesResource {
    const KEY: &'static str = "astronomical_names";
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate env_logger;

    #[test]
    fn test_fetch_resource() {
        // Init logger
        let _ = env_logger::init();

        let factory = ResourceHandler::new();
        let res = factory
            .fetch_resource::<AstronomicalNamesResource>()
            .unwrap();
        assert!(res.names.len() > 0);
        assert!(res.greek.len() > 0);
        assert!(res.decorators.len() > 0);
    }
}
