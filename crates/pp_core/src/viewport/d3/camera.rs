use cgmath::InnerSpace;

#[derive(Debug, Clone)]
pub struct Camera3D {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    /// Speed of orbiting, e.g. rotating around target
    pub speed_orbit: f32,
    /// Speed of dollying, e.g. zooming in / out
    pub speed_dolly: f32,
    /// Speed of panning, e.g. moving left / right
    pub speed_pan: f32,

    // Indicates the camera's state has changed, needing to update the uniform buffer
    pub is_dirty: bool,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            eye: (4.0, 4.0, 4.0).into(),
            target: (0.0, 0.0, 0.5).into(),
            up: cgmath::Vector3::unit_z(),
            speed_orbit: 0.005,
            speed_dolly: 0.05,
            speed_pan: 0.005,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            is_dirty: true,
        }
    }
}

impl Camera3D {
    /// Rotates the camera around the target
    pub fn orbit(&mut self, dx: f64, dy: f64) {
        let forward = self.eye - self.target; // Vector from target to eye
        let distance = forward.magnitude(); // Distance between eye and target

        // Convert to spherical coordinates
        let mut theta = forward.y.atan2(forward.x); // Azimuth angle (rotation around the vertical axis)
        let mut phi = forward.z.atan2((forward.x * forward.x + forward.y * forward.y).sqrt()); // Elevation angle

        // Adjust angles based on input deltas
        theta -= dx as f32 * self.speed_orbit; // Horizontal rotation
        phi += dy as f32 * self.speed_orbit; // Vertical rotation

        // Clamp phi to avoid flipping the camera
        let epsilon = 0.01; // To avoid gimbal lock
        phi = phi
            .clamp(-std::f32::consts::FRAC_PI_2 + epsilon, std::f32::consts::FRAC_PI_2 - epsilon);

        // Convert back to Cartesian coordinates
        let new_forward = cgmath::Vector3::new(
            distance * phi.cos() * theta.cos(), // X
            distance * phi.cos() * theta.sin(), // Y
            distance * phi.sin(),               // Z
        );

        // Update the eye position
        self.eye = self.target + new_forward;
        self.is_dirty = true
    }

    /// Dollies the camera towards / away from the target
    pub fn dolly(&mut self, delta: f64) {
        let forward = self.eye - self.target;
        self.eye -= forward * delta as f32 * self.speed_dolly; // Move the eye along the forward direction

        // Mark the camera as dirty for recalculations
        self.is_dirty = true;
    }

    /// Pans the camera by moving its target
    pub fn pan(&mut self, dx: f64, dy: f64) {
        // Calculate the right and up vectors
        let forward = (self.target - self.eye).normalize();
        let right = forward.cross(cgmath::Vector3::unit_z()).normalize();
        let up = right.cross(forward).normalize();

        // Adjust the target and eye position based on the input
        let pan_delta =
            right * (dx as f32 * self.speed_pan) + up * (-1.0 * dy as f32 * self.speed_pan);
        self.eye -= pan_delta;
        self.target -= pan_delta;

        // Mark the camera as dirty for recalculations
        self.is_dirty = true;
    }
}
