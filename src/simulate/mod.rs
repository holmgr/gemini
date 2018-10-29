mod generators;
pub mod resources;

use self::generators::generate_galaxy;
use self::resources::{fetch_resource, ShipResource};
use config::Simulation as SimulationConfig;
use core::economy::Economy;
use core::game::Game;
use player::Player;
use utils::Point;

pub struct Simulator {
    config: SimulationConfig,
}

impl Simulator {
    pub fn new(config: SimulationConfig) -> Self {
        Simulator { config }
    }

    pub fn new_game(&self) -> Game {
        let mut game_state = Game::new();

        // Generate galaxy
        info!("Generating galaxy...");
        let galaxy = generate_galaxy(&self.config);

        info!("Setting up economy...");
        game_state.economy = Economy::new(&galaxy);

        game_state.galaxy = galaxy;

        info!("Loading ships...");
        game_state
            .shipyard
            .add_ships(fetch_resource::<ShipResource>().unwrap());

        info!("Creating player...");
        game_state.player = Player::new(
            self.config.starting_credits,
            game_state.shipyard.create_base_ship(),
            // TODO: Replace starting point in config.
            game_state.galaxy.nearest(&Point::origin()).unwrap(),
        );

        game_state
    }
}
