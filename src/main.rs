#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate notify;
extern crate petgraph;
extern crate preferences;
extern crate rand;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate statrs;

extern crate env_logger;
#[macro_use(info, debug, log)]
extern crate log;

mod game_config;
mod resources;
mod generators;
mod astronomicals;

use generators::generate_galaxy;

fn main() {
    // Init logger
    let _ = env_logger::init();

    // Load GameConfig from disk
    let mut config = game_config::GameConfig::retrieve();
    info!("Initial config is: {:?}", config);

    info!("Generating galaxy...");
    let mut galaxy = generate_galaxy(&config);

    // Reload GameConfig if file on disk changes
    loop {
        match game_config::GameConfig::await_update() {
            Ok(new_config) => {
                config = new_config;
                info!(
                    "Game config updated, reloading, config is now: {:?}",
                    config
                );
                info!("Regenerating galaxy...");
                galaxy = generate_galaxy(&config);
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
