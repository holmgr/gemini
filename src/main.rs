#[macro_use]
extern crate derive_builder;
extern crate env_logger;
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
extern crate termion;
extern crate tui;

#[macro_use(info, debug, log)]
extern crate log;

mod game_config;
mod resources;
mod generators;
mod astronomicals;
mod entities;
mod game;
mod gui;

use std::sync::Arc;
use generators::generate_galaxy;

fn main() {
    // Init logger
    let _ = env_logger::init();

    // Load GameConfig from disk
    let config = game_config::GameConfig::retrieve();
    info!("Initial config is: {:#?}", config);

    // Inital game state
    let game_state = game::Game::new();

    // Generate galaxy
    if let Ok(unwrapped) = Arc::try_unwrap(game_state.clone()) {
        info!("Generating galaxy...");
        *unwrapped.galaxy.lock().unwrap() = generate_galaxy(&config);
    }

    // Init and start gui
    let mut gui = gui::Gui::new(game_state.clone());
    gui.start();
}
