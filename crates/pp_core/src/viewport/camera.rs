pub struct CameraPerspective3D {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    // Indicates the camera's state has changed, needing to update the uniform buffer
    pub is_dirty: bool,
}

impl Default for CameraPerspective3D {
    fn default() -> Self {
        Self {
            eye: (4.0, 4.0, 4.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_z(),
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            is_dirty: true,
        }
    }
}
