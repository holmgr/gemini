use super::*;

mod empty;
use self::empty::Empty;

/// Variants of UI state transtions (i.e switching between views).
enum Trans {
    None,
    Pop,
    Push(Box<dyn View>),
    Replace(Box<dyn View>),
    Quit,
}

/// Holds the state of multiple different views which can be switched.
pub struct StateMachine {
    stack: Vec<Box<dyn View>>,
}

impl StateMachine {
    /// Create a new state machine with the given view as starting view.
    pub fn new(start_view: Box<dyn View>) -> StateMachine {
        StateMachine {
            stack: vec![start_view],
        }
    }

    /// Handles the given event which can update the current view.
    /// TODO: Handle quit events properly.
    pub fn handle_event(&mut self, event: Event) {
        match self.stack.last_mut().unwrap().handle_event(event) {
            Trans::None => {}
            Trans::Pop => {
                self.stack.pop();
            }
            Trans::Push(view) => self.stack.push(view),
            Trans::Replace(view) => {
                self.stack.pop();
                self.stack.push(view)
            }
            Trans::Quit => self.stack.clear(),
        }
    }

    /// The current view, if any.
    pub fn current(&self) -> Option<&Box<dyn View>> {
        self.stack.last()
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        StateMachine::new(Box::new(Empty::new()))
    }
}

trait View {
    /// Handles the given input in the view.
    fn handle_event(&mut self, event: Event) -> Trans;

    /// Returns the current layout for this view.
    fn layout(&self) -> &Layout;
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct TestView {
        layout: Layout,
    }
    impl View for TestView {
        fn handle_event(&mut self, event: Event) -> Trans {
            match event {
                Event::MouseDown { .. } => Trans::Push(Box::new(TestView {
                    layout: LayoutBuilder::new(LayoutDirection::Horizontal).build(),
                })),
                Event::MouseUp { .. } => Trans::Pop,
                _ => Trans::None,
            }
        }

        fn layout(&self) -> &Layout {
            &self.layout
        }
    }

    #[test]
    fn test_state_pop() {
        let mut sm = StateMachine::new(Box::new(TestView {
            layout: LayoutBuilder::new(LayoutDirection::Horizontal).build(),
        }));
        assert!(sm.current().is_some());

        sm.handle_event(Event::MouseDown {
            button: MouseButton::Left,
            x: 0,
            y: 0,
        });
        assert!(sm.current().is_some());

        // Should be able to pop twice
        sm.handle_event(Event::MouseUp {
            button: MouseButton::Left,
            x: 0,
            y: 0,
        });
        assert!(sm.current().is_some());
        sm.handle_event(Event::MouseUp {
            button: MouseButton::Left,
            x: 0,
            y: 0,
        });
        assert!(sm.current().is_none());
    }
}
