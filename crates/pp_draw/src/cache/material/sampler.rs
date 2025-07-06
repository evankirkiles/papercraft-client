use pp_core::material::texture::{MinMagFilter, Sampler, WrappingMode};
use wgpu::{AddressMode, FilterMode};

use crate::gpu;

#[derive(Clone, Debug)]
pub struct SamplerGPU {
    pub sampler: wgpu::Sampler,
}

impl SamplerGPU {
    pub fn new(ctx: &gpu::Context, sampler: &Sampler) -> Self {
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: match sampler.wrap_u {
                WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
                WrappingMode::MirroredRepeat => AddressMode::MirrorRepeat,
                WrappingMode::Repeat => AddressMode::Repeat,
            },
            address_mode_v: match sampler.wrap_v {
                WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
                WrappingMode::MirroredRepeat => AddressMode::MirrorRepeat,
                WrappingMode::Repeat => AddressMode::Repeat,
            },
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: sampler
                .mag_filter
                .map(|filter| match filter {
                    MinMagFilter::Nearest => FilterMode::Nearest,
                    MinMagFilter::Linear => FilterMode::Linear,
                })
                .unwrap_or_default(),
            min_filter: sampler
                .min_filter
                .map(|filter| match filter {
                    MinMagFilter::Nearest => FilterMode::Nearest,
                    MinMagFilter::Linear => FilterMode::Linear,
                })
                .unwrap_or_default(),
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self { sampler }
    }

    pub fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }
}
