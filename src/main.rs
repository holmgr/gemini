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
extern crate toml;

mod astronomicals;
mod economy;
mod entities;
mod game;
mod game_config;
mod generators;
mod resources;
mod simulator;
mod utils;

use std::io;

use log::LevelFilter;
use simulator::Simulator;

/// Setup logging to file in user data dir.
pub fn setup_logger() -> Result<(), fern::InitError> {
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
        .chain(io::stdout())
        .apply()?;
    Ok(())
}

fn main() {
    // Init logger
    setup_logger().unwrap();

    // Start simulator
    let mut simulator = Simulator::new();

    simulator.new_game();
}
