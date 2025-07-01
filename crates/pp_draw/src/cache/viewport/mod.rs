use cutting::CuttingViewportGPU;
use folding::FoldingViewportGPU;
use pp_editor::viewport::{Viewport, ViewportContent};

use crate::gpu;

mod camera;
pub mod cutting;
pub mod folding;

#[derive(Debug, Clone, Copy)]
pub enum ViewportSyncError {
    WrongViewportType,
}

/// GPU representation of a viewport, used to set viewport in render passes
/// and supply the camera uniform for vertex shaders.
#[derive(Debug, Clone)]
pub enum ViewportGPU {
    Folding(folding::FoldingViewportGPU),
    Cutting(cutting::CuttingViewportGPU),
}

impl ViewportGPU {
    pub fn new(ctx: &gpu::Context, viewport: &Viewport) -> Self {
        match viewport.content {
            ViewportContent::Folding(_) => ViewportGPU::Folding(FoldingViewportGPU::new(ctx)),
            ViewportContent::Cutting(_) => ViewportGPU::Cutting(CuttingViewportGPU::new(ctx)),
        }
    }
}

pub trait BindableViewport {
    fn sync(
        &mut self,
        ctx: &gpu::Context,
        viewport: &mut Viewport,
    ) -> Result<(), ViewportSyncError>;
    fn bind(&self, render_pass: &mut wgpu::RenderPass);
}

impl BindableViewport for ViewportGPU {
    fn sync(
        &mut self,
        ctx: &gpu::Context,
        viewport: &mut Viewport,
    ) -> Result<(), ViewportSyncError> {
        match self {
            ViewportGPU::Folding(vp) => vp.sync(ctx, viewport),
            ViewportGPU::Cutting(vp) => vp.sync(ctx, viewport),
        }
    }

    fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        match self {
            ViewportGPU::Folding(vp) => vp.bind(render_pass),
            ViewportGPU::Cutting(vp) => vp.bind(render_pass),
        }
    }
}
