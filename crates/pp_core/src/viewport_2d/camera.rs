#[derive(Debug, Clone)]
pub struct Camera2D {
    /// The position of the camera in the XY plane
    pub eye: cgmath::Point2<f32>,
    /// The distance of the camera from the Z plane
    pub zoom: f32,

    /// Speed of dollying, e.g. zooming in / out
    pub speed_dolly: f32,
    /// Speed of panning, e.g. moving left / right
    pub speed_pan: f32,

    // Indicates the camera's state has changed, needing to update the uniform buffer
    pub is_dirty: bool,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            eye: (0.0, 0.0).into(),
            zoom: 0.5,
            speed_dolly: 0.03,
            speed_pan: 0.003,
            is_dirty: true,
        }
    }
}

impl Camera2D {
    /// Dollies the camera towards / away from the target
    pub fn dolly(&mut self, delta: f64) {
        const MIN_ZOOM: f32 = 0.1;
        const MAX_ZOOM: f32 = 10.0;
        self.zoom = (self.zoom * (1.0 + delta as f32 * self.speed_dolly)).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Pans the camera by moving its target
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.eye.x -= dx as f32 * self.speed_pan / self.zoom;
        self.eye.y += dy as f32 * self.speed_pan / self.zoom;
    }
}
