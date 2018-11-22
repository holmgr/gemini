use super::*;

/// Variants of UI state transtions (i.e switching between views).
enum Trans {
    None,
    Pop,
    Push(Box<dyn View>),
    Replace(Box<dyn View>),
    Quit
}

/// Holds the state of multiple different views which can be switched.
struct StateMachine {
    stack: Vec<Box<dyn View>>
}

impl StateMachine {
    pub fn new(start_view: Box<dyn View>) -> StateMachine {
        StateMachine {
            stack: vec![start_view]
        }
    }

    /// Handles the given event which can update the current view.
    pub fn handle_event(&mut self, event: Event) {
        match self.stack.last_mut().unwrap().handle_event(event) {
             Trans::None => {},
             Trans::Pop => { self.stack.pop(); },
             Trans::Push(view) => { self.stack.push(view) },
             Trans::Replace(view) => { self.stack.pop(); self.stack.push(view) },
             Trans::Quit => { self.stack.clear() }
        }
    }

    pub fn current(&self) -> Option<&Box<dyn View>> {
        self.stack.last()
    }
}

trait View {
    /// Updates this View.
    fn handle_event(&mut self, event: Event) -> Trans;
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct PopView {}
    impl View for PopView {
        fn handle_event(&mut self, _event: Event) -> Trans {
            Trans::Pop
        }
    }

    #[test]
    fn test_state_pop() {
        let mut sm = StateMachine::new(Box::new(PopView {}));
        assert!(sm.current().is_some());
        sm.handle_event(Event::Quit);
        assert!(sm.current().is_none());
    }
}
