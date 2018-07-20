extern crate app_dirs;
extern crate bincode;
extern crate chrono;
#[macro_use]
extern crate derive_builder;
extern crate fern;
extern crate inflector;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate notify;
extern crate petgraph;
extern crate rand;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate spade;
extern crate statrs;
extern crate termion;
extern crate textwrap;
extern crate toml;
extern crate tui;

mod astronomicals;
mod economy;
mod entities;
mod event;
mod game;
mod game_config;
mod generators;
mod gui;
mod player;
mod resources;
mod ship;
mod utils;

use app_dirs::{get_data_root, AppDataType};
use economy::Economy;
use generators::generate_galaxy;
use log::LevelFilter;

/// Setup logging to file in user data dir.
pub fn setup_logger() -> Result<(), fern::InitError> {
    let output_path = get_data_root(AppDataType::UserConfig)
        .unwrap()
        .join("gemini/debug.log");
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Off)
        .level_for("gemini", LevelFilter::Trace)
        .chain(fern::log_file(output_path)?)
        .apply()?;
    Ok(())
}

fn main() {
    // Init logger
    setup_logger().unwrap();

    // Load GameConfig from disk
    let config = game_config::GameConfig::retrieve();
    info!("Initial config is: {:#?}", config);

    // Inital game state
    let game_state = match game::Game::load() {
        Some(game_state) => game_state,
        None => {
            let game_state = game::Game::new();

            // Generate galaxy
            info!("Generating galaxy...");
            let galaxy = generate_galaxy(&config);

            info!("Setting up economy...");
            *game_state.economy.lock().unwrap() = Economy::new(&galaxy);

            *game_state.galaxy.lock().unwrap() = galaxy;

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

            game_state.update();
            game_state.save_all();
            game_state
        }
    };

    // Start event handler
    event::EventHandler::start();

    // Init and start gui
    let mut gui = gui::Gui::new(&game_state);
    gui.start();
}
