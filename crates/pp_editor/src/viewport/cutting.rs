use serde::{Deserialize, Serialize};

use super::camera::orthographic::OrthographicCamera;

/// A viewport for "cutting", meaning a 2D orthographic view of pieces, layout
/// elements like images / text, and pages.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CuttingViewport {
    pub camera: OrthographicCamera,
}
