#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate preferences;
extern crate notify;
extern crate rand;
extern crate petgraph;
extern crate rayon;

#[macro_use(info, log)]
extern crate log;
extern crate env_logger;

mod game_config;
mod resources;
mod generators;

use generators::Gen;

fn main() {
    // Init logger
    let _ = env_logger::init();

    // Load GameConfig from disk
    let mut config = game_config::GameConfig::retrieve();
    info!("Initial config is: {:?}", config);

    let fac = resources::ResourceHandler::new();
    let astro = fac.fetch_resource::<resources::AstronomicalNamesResource>()
        .unwrap();

    let mut name_gen = generators::names::NameGen::new(42);
    name_gen.train(&astro);

    for _ in 0..10 {
        println!("{:?}", name_gen.generate());
    }

    // Reload GameConfig if file on disk changes
    loop {
        match game_config::GameConfig::await_update() {
            Ok(new_config) => {
                config = new_config;
                info!(
                    "Game config updated, reloading, config is now: {:?}",
                    config
                );
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
