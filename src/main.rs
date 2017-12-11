#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate preferences;
extern crate notify;
extern crate rand;
extern crate petgraph;
extern crate rayon;
extern crate nalgebra;
extern crate statrs;
#[macro_use]
extern crate lazy_static;

#[macro_use(info, debug, log)]
extern crate log;
extern crate env_logger;

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
    debug!("Generated galaxy: {:?}", galaxy);

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
                debug!("Generated galaxy: {:?}", galaxy);
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
