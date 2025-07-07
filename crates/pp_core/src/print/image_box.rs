use crate::{measures::Rect, ImageId};

#[derive(Debug, Clone)]
pub struct ImageBox {
    /// The bounds of the textbox
    pub bounds: Rect<f32>,
    /// The actual image
    pub image: ImageId,
}
