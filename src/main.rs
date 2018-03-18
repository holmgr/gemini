#[macro_use]
extern crate derive_builder;
extern crate env_logger;
extern crate inflector;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate notify;
extern crate petgraph;
extern crate preferences;
extern crate rand;
extern crate rayon;
extern crate serde;
extern crate serde_cbor;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate spade;
extern crate statrs;
extern crate termion;
extern crate toml;
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
mod event;
mod ship;
mod utils;
mod player;

use generators::generate_galaxy;

fn main() {
    // Init logger
    let _ = env_logger::init();

    // Load GameConfig from disk
    let config = game_config::GameConfig::retrieve();
    info!("Initial config is: {:#?}", config);

    let enable_gui = config.enable_gui;

    // Inital game state
    let game_state = match game::Game::load() {
        Some(game_state) => game_state,
        None => {
            let game_state = game::Game::new();

            // Generate galaxy
            info!("Generating galaxy...");
            *game_state.galaxy.lock().unwrap() = generate_galaxy(&config);

            info!("Loading ships...");
            game_state
                .shipyard
                .lock()
                .unwrap()
                .add_ships(resources::fetch_resource::<resources::ShipResource>().unwrap());

            info!("Creating player...");
            *game_state.player.lock().unwrap() = player::Player::new(
                config.starting_credits,
                game_state.shipyard.lock().unwrap().create_base_ship(),
                // TODO: Replace starting point in config.
                game_state
                    .galaxy
                    .lock()
                    .unwrap()
                    .nearest(&utils::Point::origin())
                    .unwrap(),
            );

            game_state.save_all();
            game_state
        }
    };

    // Start event handler
    event::EventHandler::start();

    // Init and start gui
    if enable_gui {
        let mut gui = gui::Gui::new(game_state);
        gui.start();
    }
}
