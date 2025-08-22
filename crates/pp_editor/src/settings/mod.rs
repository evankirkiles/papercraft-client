use serde::Serialize;
use tsify::Tsify;

use theme::Theme;

pub mod theme;

/// Settings represent user-specific configurations of the editor.
#[derive(Debug, Tsify, Serialize)]
pub struct Settings {
    pub theme: Theme,
    /// Have settings changed?
    pub is_dirty: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { theme: Default::default(), is_dirty: true }
    }
}
