use raui_core::{layout::Layout, renderer::Renderer, widget::unit::WidgetUnit};

#[derive(Debug, Default, Copy, Clone)]
pub struct BinaryRenderer;

impl Renderer<Vec<u8>, bincode::Error> for BinaryRenderer {
    fn render(&mut self, tree: &WidgetUnit, _layout: &Layout) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(tree)
    }
}
