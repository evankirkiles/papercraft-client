use crate::measures::Dimensions;

pub mod orthographic;
pub mod perspective;

pub trait Camera {
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool);
    fn eye(&self) -> [f32; 4];
    fn view_proj(&self, dims: Dimensions<f32>) -> cgmath::Matrix4<f32>;
}
