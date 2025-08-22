use crate::cache::{
    material::MaterialGPU,
    mesh::piece::PieceGPU,
    print::PrintLayoutGPU,
    settings::SettingsGPU,
    tool::{rotate::RotateToolGPU, select_box::SelectBoxToolGPU, translate::TranslateToolGPU},
    viewport::ViewportGPU,
};

/// Global ordering of bind groups, so shaders can refer to consistent bind
/// groups without conflict.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum BindGroup {
    Settings,
    Viewport,
    Piece,
    PrintLayout,
    Material,
    Tool,
}

impl BindGroup {
    pub fn value(&self) -> u32 {
        match self {
            BindGroup::Settings => 0,
            BindGroup::Viewport => 1,
            // Mesh path
            BindGroup::Piece => 2,
            BindGroup::Material => 3,
            // Tool Path
            BindGroup::Tool => 2,
            // Print path
            BindGroup::PrintLayout => 2,
        }
    }
}
/// Each tool has its own uniform layout
#[derive(Debug)]
pub struct ToolBindGroupLayouts {
    pub select_box: wgpu::BindGroupLayout,
    pub rotate: wgpu::BindGroupLayout,
    pub translate: wgpu::BindGroupLayout,
}

/// Shared BindGroup layouts created at the start of the program, allowing
/// pipelines to re-use Bind Groups without creating wholly new layouts.
#[derive(Debug)]
pub struct SharedBindGroupLayouts {
    pub settings: wgpu::BindGroupLayout,
    pub viewport: wgpu::BindGroupLayout,
    pub piece: wgpu::BindGroupLayout,
    pub material: wgpu::BindGroupLayout,
    pub print_layout: wgpu::BindGroupLayout,
    pub tool: ToolBindGroupLayouts,
}

impl SharedBindGroupLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            settings: SettingsGPU::create_bind_group_layout(device),
            viewport: ViewportGPU::create_bind_group_layout(device),
            piece: PieceGPU::create_bind_group_layout(device),
            material: MaterialGPU::create_bind_group_layout(device),
            print_layout: PrintLayoutGPU::create_bind_group_layout(device),
            tool: ToolBindGroupLayouts {
                select_box: SelectBoxToolGPU::create_bind_group_layout(device),
                rotate: RotateToolGPU::create_bind_group_layout(device),
                translate: TranslateToolGPU::create_bind_group_layout(device),
            },
        }
    }
}
