pub mod camera;

/// Viewports represent splits of the surface which each render a different
/// view into the scene.
#[derive(Debug, Clone)]
pub struct Viewport2D {
    pub camera: camera::Camera2D,
}

impl Default for Viewport2D {
    /// The default viewport is full
    fn default() -> Self {
        Self { camera: Default::default() }
    }
}
