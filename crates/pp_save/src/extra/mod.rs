use serde::{Deserialize, Serialize};

use crate::extra::{cut::CutPrimitiveAttributes, piece::PiecePrimitiveAttributes};

pub(super) mod cut;
pub(super) mod page;
pub(super) mod piece;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MeshExtras {
    /// Edges in the 3D geometry which are marked as "cut"
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub papercraft: Option<PapercraftMeshExtra>,
}

/// Custom data for the papercraft unfolding system, stored inside the save file
/// GLTF under the `extras` attribute of any unfolded `mesh`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PapercraftMeshExtra {
    /// Edges in the 3D geometry which are marked as "cut"
    pub cuts: CutPrimitiveAttributes,
    /// Pieces in the 3D geometry, indicated by a "root" face
    pub pieces: PiecePrimitiveAttributes,
}

/// Custom data for the papercraft unfolding system, stored inside the save file
/// GLTF under the `extras` attribute at the `root`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PapercraftExtra {
    /// An array of pages (used in the print layout).
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pages: Vec<page::SavePage>,
}
