use serde::Serialize;
use tsify::Tsify;

use theme::Theme;

pub mod theme;

/// Long-term user preferences that persist across sessions.
#[derive(Debug, Tsify, Serialize)]
pub struct Preferences {
    pub theme: Theme,
    /// Have preferences changed, requiring a GPU re-upload?
    pub is_dirty: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self { theme: Default::default(), is_dirty: true }
    }
}
