pub mod camera;

/// Viewports represent splits of the surface which each render a different
/// view into the scene.
#[derive(Debug, Clone)]
pub struct Viewport3D {
    pub camera: camera::Camera3D,
}

impl Default for Viewport3D {
    /// The default viewport is full
    fn default() -> Self {
        Self { camera: Default::default() }
    }
}
