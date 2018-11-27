use ggez::{
    event::*,
    graphics::{Drawable, Point2, Vector2},
    *,
};

use core::game::Game;
mod component;
mod layout;
mod render;
mod view;

use self::component::Component;
use self::layout::{Layout, LayoutBuilder, LayoutDirection};
use self::render::{RenderArea, RenderContext, Renderable};
use self::view::StateMachine;

/// UI user interaction events.
enum Event {
    MouseDown {
        button: MouseButton,
        x: i32,
        y: i32,
    },
    MouseUp {
        button: MouseButton,
        x: i32,
        y: i32,
    },
    MouseMove {
        x: i32,
        y: i32,
        xrel: i32,
        yrel: i32,
    },
    MouseWheel {
        x: i32,
        y: i32,
    },
    KeyDown {
        code: Keycode,
        repeat: bool,
    },
    KeyUp {
        code: Keycode,
        repeat: bool,
    },
}

/// UI handler for dispatching events and holding main state.
pub struct UI {
    game_state: Game,
    states: StateMachine,
    frames: u64,
}

impl UI {
    /// Create a new UI.
    pub fn new(game_state: Game) -> Self {
        UI {
            frames: 0,
            game_state,
            states: StateMachine::default(),
        }
    }

    /// Start the UI, i.e the event loop and rendering.
    pub fn start(&mut self) {
        let c = conf::Conf::new();
        let ctx = &mut Context::load_from_conf("gemini", "holmgr", c).unwrap();
        event::run(ctx, self).unwrap();
    }
}

impl event::EventHandler for UI {
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        self.states.handle_event(Event::MouseDown { button, x, y });
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        self.states.handle_event(Event::MouseUp { button, x, y });
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        _state: MouseState,
        x: i32,
        y: i32,
        xrel: i32,
        yrel: i32,
    ) {
        self.states
            .handle_event(Event::MouseMove { x, y, xrel, yrel });
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: i32, y: i32) {
        self.states.handle_event(Event::MouseWheel { x, y });
    }

    fn key_down_event(&mut self, _ctx: &mut Context, code: Keycode, _keymod: Mod, repeat: bool) {
        self.states.handle_event(Event::KeyDown { code, repeat });
    }

    fn key_up_event(&mut self, _ctx: &mut Context, code: Keycode, _keymod: Mod, repeat: bool) {
        self.states.handle_event(Event::KeyUp { code, repeat });
    }

    /// Update the UI.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    /// Draw the UI.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        graphics::present(ctx);
        if self.frames % 100 == 0 {
            debug!("FPS: {:.1}", timer::get_fps(ctx));
        }
        self.frames += 1;

        timer::yield_now();
        Ok(())
    }
}
