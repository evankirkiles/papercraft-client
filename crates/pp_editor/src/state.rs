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

/// The persistent selection gesture the user is in. Unlike the transient
/// gizmo tools, this outlives any single interaction: it is switched by keybind
/// (`C` / `Esc`) or from the tool column in the UI.
#[wasm_bindgen]
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
pub enum SelectTool {
    /// Drag a rectangle to select everything inside it
    #[default]
    Box,
    /// Paint over elements with a circular brush to select them
    Paint,
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
    /// The current selection gesture (box drag or brush paint)
    pub select_tool: SelectTool,
    /// The fold-progress scalar, 0.0-1.0, driving the folding animation
    pub t: f32,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            is_xray: false,
            is_presentation: false,
            selection_mode: Default::default(),
            select_tool: Default::default(),
            t: 1.0,
        }
    }
}
