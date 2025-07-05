use serde::{Deserialize, Serialize};

use super::camera::perspective::PerspectiveCamera;

/// A viewport for "folding", meaning a 3D perspective view of the finalized model.
/// Note that this view also supports rendering the 2D pieces from the "cutting" view
/// so we can tween between the folded / unfolded state.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FoldingViewport {
    pub camera: PerspectiveCamera,
}
