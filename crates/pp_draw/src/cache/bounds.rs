use pp_core::bounds::Aabb3;

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

/// How much larger than the model's half-extent the folding viewport's
/// circular grid should be, so there's a bit of visible padding around it.
const GRID_RADIUS_PADDING_FACTOR: f32 = 1.5;
/// The smallest radius the grid will shrink to, so an empty or tiny scene
/// still shows a reasonably sized grid.
const GRID_RADIUS_MIN: f32 = 4.0;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoundsUniform {
    /// xyz = world-space min corner, w = the folding grid's fitted radius,
    /// precomputed here so the WGSL grid shader doesn't need to duplicate
    /// the extent/padding/floor logic in both its vertex and fragment stages.
    /// Only the grid's *extent* scales with the model — cell size (i.e. how
    /// many real-world units a grid square represents) stays fixed, so the
    /// grid always reads at the same physical scale.
    min: [f32; 4],
    /// xyz = world-space max corner, w unused.
    max: [f32; 4],
}

impl BoundsUniform {
    fn from(aabb: &Aabb3) -> Self {
        let extent = aabb.size();
        let half_extent = extent.x.max(extent.y).max(extent.z) * 0.5;
        let radius = (half_extent * GRID_RADIUS_PADDING_FACTOR).max(GRID_RADIUS_MIN);
        Self {
            min: [aabb.min.x, aabb.min.y, aabb.min.z, radius],
            max: [aabb.max.x, aabb.max.y, aabb.max.z, 0.0],
        }
    }

    pub fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            count: None,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }
    }
}

/// Tracks the document's world-space bounding box on the GPU, shared by the
/// folding viewport's grid (auto-fit radius) and the bounding-box wireframe.
#[derive(Debug)]
pub struct BoundsGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,
    last: Option<Aabb3>,
}

impl BoundsGPU {
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bounds"),
            entries: &[BoundsUniform::bind_group_layout_entry(0)],
        })
    }

    pub fn new(ctx: &gpu::Context) -> Self {
        let buf = gpu::UniformBuf::new(ctx, "bounds".to_string(), size_of::<BoundsUniform>());
        Self {
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bounds"),
                layout: &ctx.shared.bind_group_layouts.bounds,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() }],
            }),
            buf,
            last: None,
        }
    }

    /// Recomputes the document's world bounds and re-uploads the uniform only
    /// if they've changed since the last frame. Returns the current bounds so
    /// callers (e.g. the bbox wireframe) can rebuild CPU-side geometry.
    pub fn prepare(&mut self, ctx: &gpu::Context, state: &pp_core::State) -> Aabb3 {
        let aabb = state.world_bounds();
        if self.last != Some(aabb) {
            self.buf.update(ctx, &[BoundsUniform::from(&aabb)]);
            self.last = Some(aabb);
        }
        aabb
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(BindGroup::Bounds.value(), &self.bind_group, &[]);
    }
}
