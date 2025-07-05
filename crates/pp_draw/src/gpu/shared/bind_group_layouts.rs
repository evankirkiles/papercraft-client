use crate::cache::{
    material::MaterialGPU,
    mesh::piece::PieceGPU,
    tool::{rotate::RotateToolGPU, select_box::SelectBoxToolGPU},
    viewport::{bounds::ViewportBoundsGPU, camera::CameraGPU},
};

/// Global ordering of bind groups, so shaders can refer to consistent bind
/// groups without conflict.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum BindGroup {
    Viewport,
    Camera,
    Piece,
    Material,
    Tool,
}

impl BindGroup {
    pub fn value(&self) -> u32 {
        match self {
            BindGroup::Viewport => 0,
            BindGroup::Camera => 1,
            // Tool Path
            BindGroup::Tool => 2,
            // Mesh path
            BindGroup::Piece => 2,
            BindGroup::Material => 3,
        }
    }
}
/// Each tool has its own uniform layout
#[derive(Debug)]
pub struct ToolBindGroupLayouts {
    pub select_box: wgpu::BindGroupLayout,
    pub rotate: wgpu::BindGroupLayout,
}

/// Shared BindGroup layouts created at the start of the program, allowing
/// pipelines to re-use Bind Groups without creating wholly new layouts.
#[derive(Debug)]
pub struct SharedBindGroupLayouts {
    pub viewport: wgpu::BindGroupLayout,
    pub camera: wgpu::BindGroupLayout,
    pub piece: wgpu::BindGroupLayout,
    pub material: wgpu::BindGroupLayout,
    pub tool: ToolBindGroupLayouts,
}

impl SharedBindGroupLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            viewport: ViewportBoundsGPU::create_bind_group_layout(device),
            camera: CameraGPU::create_bind_group_layout(device),
            piece: PieceGPU::create_bind_group_layout(device),
            material: MaterialGPU::create_bind_group_layout(device),
            tool: ToolBindGroupLayouts {
                select_box: SelectBoxToolGPU::create_bind_group_layout(device),
                rotate: RotateToolGPU::create_bind_group_layout(device),
            },
        }
    }
}
