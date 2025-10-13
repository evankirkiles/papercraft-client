use pp_core::State;
use serde::{Deserialize, Serialize};

/// Represents a print page
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SavePage {
    /// Page width in millimeters
    pub width: f32,
    /// Page height in millimeters
    pub height: f32,
}

/// Converts cut edges from pp_core State to PPR format
pub fn save_pages(state: &State) -> Vec<SavePage> {
    todo!()
}

/// Converts cut edges from pp_core State to PPR format
pub fn load_pages(state: &mut State, pages: &Vec<SavePage>) {
    todo!()
}
