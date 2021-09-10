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
extern crate anyhow;
extern crate clap;
extern crate serde_json;
extern crate spade;
extern crate statrs;
extern crate toml;

mod astronomicals;
mod config;
mod economy;
mod entities;
mod game;
mod generators;
mod resources;
mod simulator;
mod utils;

use anyhow::Result;
use clap::{AppSettings, Clap};
use log::LevelFilter;
use std::{fs::File, io, io::Read};

fn main() {
    // Init logger
    setup_logger().unwrap();

    // Parse the command line.
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::NewGame(t) => {
            // Display title.
            let title = include_str!("../res/title.txt");
            println!("{}", title);

            // Gets a value for config if supplied by user, or defaults to "genconfig.toml"
            let config = match parse_config(&t.config_path) {
                Ok(config) => config,
                Err(e) => {
                    warn!(
                        "Failed to get specified config at {}: due to {}. Using default",
                        &t.config_path, e
                    );
                    config::GameConfig::default()
                }
            };

            // Start simulator
            let mut simulator = simulator::Simulator::new(config);

            simulator.new_game();
        }
    }

    // TODO: Implement the rest of the program.
}

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Viktor H. <viktor.holmgren@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(version = "1.3", author = "Viktor H. <viktor.holmgren@gmail.com>")]
    NewGame(NewGame),
    //TODO: Add additional subcommands; serve (for server) etc.
}

/// Subcommand for generating a new world.
#[derive(Clap)]
struct NewGame {
    #[clap(short, long, default_value = "genconfig.toml")]
    config_path: String,
}

/// Try parse the Generation Config at the specified path.
fn parse_config(path: &str) -> Result<config::GameConfig> {
    let mut file = File::open(&path)?;

    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    let config: config::GameConfig = toml::from_str(&file_content)?;
    Ok(config)
}

pub fn setup_logger() -> Result<()> {
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
