pub mod rotate;
pub mod select_box;
pub mod translate;

pub use rotate::RotateTool;
pub use select_box::SelectBoxTool;
pub use translate::TranslateTool;

use crate::measures::Dimensions;

/// An error indicating a tool could not be created.
#[derive(Debug, Clone, Copy)]
pub enum ToolCreationError {
    NoSelection,
}

/// Common fields provided by the controller which all tools may need
#[derive(Debug, Default, Clone, Copy)]
pub struct ToolContext {
    /// The raw pixel dimensions of the viewport this tool operates on
    pub viewport: Dimensions<f32>,
    /// The Device Pixel Ratio, the number of pixels per logical pixel
    pub dpr: f32,
}

#[derive(Debug, Clone)]
pub enum Tool {
    Translate(translate::TranslateTool),
    Rotate(rotate::RotateTool),
    SelectBox(select_box::SelectBoxTool),
}

impl serde::Serialize for Tool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant_name = match self {
            Tool::Translate(_) => "Translate",
            Tool::Rotate(_) => "Rotate",
            Tool::SelectBox(_) => "SelectBox",
        };
        serializer.serialize_str(variant_name)
    }
}
