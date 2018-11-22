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
extern crate clap;
extern crate failure;
extern crate ggez;
extern crate git2;
extern crate serde_json;
extern crate spade;
extern crate statrs;
extern crate toml;

mod config;
mod core;
mod data;
mod player;
mod simulate;
mod ui;
mod utils;

use app_dirs::{get_data_root, AppDataType};
use clap::{App, Arg, SubCommand};
use config::Config;
use data::DataService;
use failure::Error;
use log::LevelFilter;
use simulate::Simulator;

/// Setup logging to file in user data dir.
pub fn setup_logger(to_stdout: bool) -> Result<(), fern::InitError> {
    let output_path = get_data_root(AppDataType::UserConfig)
        .unwrap()
        .join("gemini/debug.log");
    let mut logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        }).level(log::LevelFilter::Off)
        .level_for("gemini", LevelFilter::Trace)
        .chain(fern::log_file(output_path)?);

    if to_stdout {
        logger = logger.chain(std::io::stdout());
    }
    logger.apply()?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let matches = App::new("Gemini")
        .version("0.1")
        .author("Viktor Holmgren <viktor.holmgren@gmail.com>")
        .about("Procedurally generated space RPG")
        .arg(
            Arg::with_name("debug")
                .short("d")
                .help("Print debug information to stdout"),
        ).subcommand(
            SubCommand::with_name("start")
                .about("Start the game client")
                .version("0.1")
                .author("Viktor Holmgren <viktor.holmgren@gmail.com>"),
        ).subcommand(
            SubCommand::with_name("simulator")
                .about("Runs the simulator")
                .version("0.1")
                .author("Viktor Holmgren <viktor.holmgren@gmail.com>"),
        ).subcommand(
            SubCommand::with_name("new")
                .about("Creates a new world by running the simulator for initial state")
                .version("0.1")
                .author("Viktor Holmgren <viktor.holmgren@gmail.com>"),
        ).get_matches();

    // Setup logging.
    let debug_to_stdout = matches.is_present("debug");
    // Init logger
    setup_logger(debug_to_stdout).unwrap();

    // Load config on compile.
    let config: Config =
        toml::from_str(include_str!("../Config.toml")).expect("Failed to load config");

    let data_service = DataService::new(config.data)?;

    match matches.subcommand() {
        ("start", Some(_)) => {
            debug!("Starting client");
            let mut game = data_service.try_load()?;
            let mut ui = ui::UI::new(game);
            ui.start();
        }
        ("simulator", Some(_)) => {
            debug!("Starting simulator");
            let mut game = data_service.try_load()?;
            if let Some(days) = game.attempt_advance_time() {
                debug!("Simulated {} days", days);
                data_service.store(&game)?;
                data_service.sync_up(&format!("Simulated: {} days forward", days))?;
            }
        }
        ("new", Some(_)) => {
            // Safe since its required.
            debug!("Creating new game");
            let simulator = Simulator::new(config.simulation);
            let mut game = simulator.new_game();
            game.attempt_advance_time();

            data_service.store(&game)?;
            data_service.sync_up("Initial world simulation")?;
        }
        _ => {}
    };

    Ok(())
}
