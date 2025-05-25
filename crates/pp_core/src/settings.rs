#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub enum SelectionMode {
    Vert,
    Edge,
    #[default]
    Face,
    Piece,
}

/// Editor-wide settings
#[derive(Debug, Clone)]
pub struct Settings {
    pub selection_mode: SelectionMode,
    pub viewport_split_x: f64,
    pub viewport_split_y: f64,
    pub t: f32,
    pub is_dirty: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            selection_mode: Default::default(),
            viewport_split_x: 0.5,
            viewport_split_y: 1.0,
            t: 0.0,
            is_dirty: false,
        }
    }
}
