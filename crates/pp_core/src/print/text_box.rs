use crate::measures::Rect;

#[derive(Debug, Clone)]
pub struct TextBox {
    /// The bounds of the textbox
    pub bounds: Rect<f32>,
    /// The text content of the textbox
    pub content: String,
}
