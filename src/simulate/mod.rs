use std::sync::Arc;

mod config;
mod generators;
pub mod resources;

use self::config::GameConfig;
use self::generators::generate_galaxy;
use self::resources::{fetch_resource, ShipResource};
use core::economy::Economy;
use core::game::Game;
use player::Player;
use utils::Point;

pub struct Simulator {
    game_state: Option<Arc<Game>>,
    game_config: GameConfig,
}

impl Simulator {
    pub fn new() -> Self {
        // Load GameConfig from disk
        let game_config = GameConfig::retrieve();
        info!("Initial config is: {:#?}", game_config);
        Simulator {
            game_config,
            game_state: None,
        }
    }

    pub fn new_game(&mut self) -> Arc<Game> {
        let game_state = Game::new();

        // Generate galaxy
        info!("Generating galaxy...");
        let galaxy = generate_galaxy(&self.game_config);

        info!("Setting up economy...");
        *game_state.economy.lock().unwrap() = Economy::new(&galaxy);

        *game_state.galaxy.lock().unwrap() = galaxy;

        info!("Loading ships...");
        game_state
            .shipyard
            .lock()
            .unwrap()
            .add_ships(fetch_resource::<ShipResource>().unwrap());

        info!("Creating player...");
        *game_state.player.lock().unwrap() = Player::new(
            self.game_config.starting_credits,
            game_state.shipyard.lock().unwrap().create_base_ship(),
            // TODO: Replace starting point in config.
            game_state
                .galaxy
                .lock()
                .unwrap()
                .nearest(&Point::origin())
                .unwrap(),
        );

        game_state.update();
        game_state.save_all();
        self.game_state = Some(game_state.clone());
        game_state
    }

    pub fn load_game(&mut self) -> Option<Arc<Game>> {
        self.game_state = Game::load();
        self.game_state.clone()
    }
}
