use std::mem;

use image::ImageGPU;
use pp_core::{material::Material, ImageId, SamplerId, TextureId};
use sampler::SamplerGPU;
use slotmap::SecondaryMap;
use texture::TextureGPU;

use crate::gpu::{self, shared::bind_group_layouts::BindGroup};

pub mod image;
pub mod sampler;
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

    pub fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            count: None,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }
    }
}

#[derive(Debug)]
pub struct MaterialGPU {
    buf: gpu::UniformBuf,
    bind_group: wgpu::BindGroup,
}

impl MaterialGPU {
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material"),
            entries: &[
                MaterialUniform::bind_group_layout_entry(0), // All material config
                ImageGPU::bind_group_layout_entry(1),        // Diffuse texture: view
                SamplerGPU::bind_group_layout_entry(2),      // Diffuse texture: sampler
            ],
        })
    }

    pub fn new(
        ctx: &gpu::Context,
        mat: &Material,
        textures: &SecondaryMap<TextureId, TextureGPU>,
        images: &SecondaryMap<ImageId, ImageGPU>,
        samplers: &SecondaryMap<SamplerId, SamplerGPU>,
    ) -> Self {
        let buf = gpu::UniformBuf::new(ctx, mat.label.clone(), mem::size_of::<MaterialUniform>());
        let tex_diffuse = textures.get(mat.base_color_texture).unwrap();
        Self {
            bind_group: ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(mat.label.as_str()),
                layout: &ctx.shared.bind_group_layouts.material,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: buf.binding_resource() },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &images.get(tex_diffuse.image).unwrap().view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(
                            &samplers.get(tex_diffuse.sampler).unwrap().sampler,
                        ),
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
