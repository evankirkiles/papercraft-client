mod tool_transform;

pub use tool_transform::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct PhysicalDimensions<T> {
    pub width: T,
    pub height: T,
}

/// Common fields provided by the controller which all tools may need
#[derive(Debug, Default, Clone, Copy)]
pub struct ToolContext {
    /// The raw pixel dimensions of the viewport this tool operates on
    pub viewport: PhysicalDimensions<f32>,
    /// The Device Pixel Ratio, the number of pixels per logical pixel
    pub dpr: f32,
}

impl std::ops::Mul<PhysicalDimensions<f32>> for PhysicalDimensions<f32> {
    type Output = PhysicalDimensions<f32>;

    fn mul(self, rhs: PhysicalDimensions<f32>) -> PhysicalDimensions<f32> {
        Self { width: self.width * rhs.width, height: self.height * rhs.height }
    }
}
