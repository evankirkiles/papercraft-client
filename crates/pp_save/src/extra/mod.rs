use serde::{Deserialize, Serialize};

pub(super) mod cut;
pub(super) mod page;
pub(super) mod piece;

/// Custom data for the papercraft unfolding system, stored inside the save file
/// GLTF under the `extras` attribute.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PapercraftExtra {
    /// Edges in the 3D geometry which are marked as "cut"
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cuts: Vec<cut::Cut>,

    /// An array of pages (used in the print layout).
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pages: Vec<page::Page>,

    /// Pieces in the 3D geometry, indicated by a "root" face
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pieces: Vec<piece::Piece>,
}
