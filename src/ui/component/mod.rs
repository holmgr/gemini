use super::*;

/// A renderable component.
pub trait Component {
    fn render(&self, area: RenderArea, ctx: &mut RenderContext);
}

impl<T> Renderable for T
where T: Component {
    fn render(&self, area: RenderArea, ctx: &mut RenderContext) {
        Component::render(self, area, ctx);
    }
}
