use super::*;

/// The direction of layout.
pub enum LayoutDirection {
    Vertical,
    Horizontal
}

/// Builder for easily constructing complex tree like layouts.
pub struct LayoutBuilder {
    direction: LayoutDirection,
    children: Vec<(Box<dyn Renderable>, f64)>
}

impl LayoutBuilder {

    /// Create a new builder with the given direction.
    pub fn new(direction: LayoutDirection) -> Self {
        LayoutBuilder { direction, children: vec![] }
    }

    /// Adds a new child component/layout to this layout.
    pub fn with_child(mut self, size: f64, child: Box<dyn Renderable>) -> Self {
        self.children.push((child, size));
        self
    }
}

pub struct Layout {}

impl Renderable for Layout {}
