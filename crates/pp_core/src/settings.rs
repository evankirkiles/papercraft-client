use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub enum SelectionMode {
    #[default]
    Vert,
    Edge,
    Face,
    Piece,
}

/// Editor-wide settings
#[derive(Debug, Clone)]
pub struct Settings {
    pub selection_mode: SelectionMode,
    pub t: f32,
    pub is_dirty: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { selection_mode: Default::default(), t: 0.0, is_dirty: false }
    }
}
