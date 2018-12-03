use super::*;

/// The direction of layout.
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

/// Builder for easily constructing complex tree like layouts.
pub struct LayoutBuilder {
    direction: LayoutDirection,
    children: Vec<(Box<dyn Renderable>, f32)>,
}

impl LayoutBuilder {
    /// Create a new builder with the given direction.
    pub fn new(direction: LayoutDirection) -> Self {
        LayoutBuilder {
            direction,
            children: vec![],
        }
    }

    /// Adds a new child component/layout to this layout.
    pub fn with_child(mut self, size: f32, child: Box<dyn Renderable>) -> Self {
        self.children.push((child, size));
        self
    }

    /// Construct the layout.
    pub fn build(self) -> Layout {
        Layout {
            direction: self.direction,
            children: self.children,
        }
    }
}

/// A split based layout of components/layouts.
pub struct Layout {
    direction: LayoutDirection,
    children: Vec<(Box<dyn Renderable>, f32)>,
}

impl Renderable for Layout {
    fn render(&self, area: RenderArea, context: &mut RenderContext) {
        let mut offset = 0.;
        for (child, size) in &self.children {
            let sub_area = match self.direction {
                LayoutDirection::Horizontal => {
                    let mut sub_area = area.clone();
                    sub_area.translate(Vector2::new(offset, 0.));
                    sub_area.scale(0., *size);
                    sub_area
                }
                LayoutDirection::Vertical => {
                    let mut sub_area = area.clone();
                    sub_area.translate(Vector2::new(0., offset));
                    sub_area.scale(0., *size);
                    sub_area
                }
            };
            offset += size;
        }
    }
}
