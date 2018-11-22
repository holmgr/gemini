use core::game::Game;
use ggez::*;

/// UI handler for dispatching events and holding main state.
pub struct UI {
    game_state: Game,
}

impl UI {
    /// Create a new UI.
    pub fn new(game_state: Game) -> Self {
        UI { game_state }
    }

    /// Start the UI, i.e the event loop and rendering.
    pub fn start(&mut self) {
        let c = conf::Conf::new();
        let ctx = &mut Context::load_from_conf("gemini", "holmgr", c).unwrap();
        event::run(ctx, self).unwrap();
    }
}

impl event::EventHandler for UI {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::present(ctx);
        Ok(())
    }
}
