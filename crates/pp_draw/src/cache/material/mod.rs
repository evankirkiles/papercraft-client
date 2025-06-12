use std::{collections::HashMap, mem};

use pp_core::{id, material::Material};
use texture::TextureGPU;

use crate::gpu::{self, layouts::bind_groups::BindGroup};

pub mod image;
pub mod texture;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    base_color_factor: [f32; 4],
}

impl Default for MaterialUniform {
    fn default() -> Self {
        Self { base_color_factor: [1.0, 1.0, 1.0, 1.0] }
    }
}

impl MaterialUniform {
    fn new(piece: &pp_core::material::Material) -> Self {
        Self { base_color_factor: piece.base_color_factor }
    }
}

#[derive(Debug)]
pub struct MaterialGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,
}

impl MaterialGPU {
    pub fn new(
        ctx: &gpu::Context,
        mat: &Material,
        textures: &HashMap<id::TextureId, TextureGPU>,
    ) -> Self {
        let buf = gpu::UniformBuf::new(ctx, mat.label.clone(), mem::size_of::<MaterialUniform>());
        let tex_diffuse = textures.get(&mat.base_color_texture).unwrap();
        Self {
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(mat.label.as_str()),
                layout: &ctx.shared_layouts.bind_groups.material,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&tex_diffuse.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&tex_diffuse.sampler),
                    },
                ],
            }),
            buf,
        }
    }

    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(BindGroup::Material.value(), &self.bind_group, &[]);
    }
}
