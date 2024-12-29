pub mod camera;

/// Viewports represent splits of the surface which each render a different
/// view into the scene.
pub struct Viewport {
    pub camera: camera::CameraPerspective3D,

    /// The fraction of the window's width this viewport takes up
    pub width_frac: f32,
}

impl Default for Viewport {
    /// The default viewport is full
    fn default() -> Self {
        Self { camera: Default::default(), width_frac: 1.0 }
    }
}
