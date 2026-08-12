use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default, Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub enum SelectionMode {
    #[default]
    Vert,
    Edge,
    Face,
    Piece,
}

/// Transient interaction state which drives rendering but doesn't persist
/// across sessions, changing directly in response to user input / keybinds.
#[derive(Debug, Tsify, Serialize)]
pub struct EditorState {
    /// Is the editor in "x-ray" mode?
    pub is_xray: bool,
    /// Is the editor in "presentation" mode?
    pub is_presentation: bool,
    /// The current selection granularity (vertex, edge, face, or piece)
    pub selection_mode: SelectionMode,
    /// The fold-progress scalar, 0.0-1.0, driving the folding animation
    pub t: f32,
}

impl Default for EditorState {
    fn default() -> Self {
        Self { is_xray: false, is_presentation: false, selection_mode: Default::default(), t: 1.0 }
    }
}
