use std::sync::Arc;

use config::GameConfig;
use economy::Economy;
use game::Game;
use generators::generate_galaxy;

pub struct Simulator {
    game_state: Option<Arc<Game>>,
    game_config: GameConfig,
}

impl Simulator {
    pub fn new(config: GameConfig) -> Self {
        info!("Initial config is: {:#?}", config);
        Simulator {
            game_config: config,
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

        game_state.update();
        self.game_state = Some(game_state.clone());
        game_state
    }
}
