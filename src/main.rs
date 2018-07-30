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
mod simulator;
mod utils;

use app_dirs::{get_data_root, AppDataType};
use log::LevelFilter;
use simulator::Simulator;

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

    // Start event handler
    event::EventHandler::start();

    // Start simulator
    let simulator = Simulator::new();

    // Init and start gui
    let mut gui = gui::Gui::new(simulator);
    gui.start();
}
