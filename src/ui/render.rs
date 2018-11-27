use super::*;

use ggez::graphics::{Drawable, Rect};

/// Area which the rendering should take place in.
pub type RenderArea = Rect;

/// Context used for rendering.
pub struct RenderContext<'a> {
    game_state: &'a Game,
}

impl<'a> RenderContext<'a> {
    /// Creates a new context.
    pub fn new(game_state: &'a Game) -> Self {
        RenderContext { game_state }
    }

    /// Retrives the game state.
    pub fn game_state(&self) -> &Game {
        self.game_state
    }
}

/// A renderable type.
pub trait Renderable {
    fn render(&self, area: RenderArea, context: &RenderContext) -> Vec<Box<Drawable>>;
}
