use pp_core::material::texture::{MinMagFilter, Texture, WrappingMode};
use wgpu::{AddressMode, FilterMode};

use crate::gpu;

use super::image::ImageGPU;

#[derive(Clone, Debug)]
pub struct TextureGPU {
    pub label: String,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl TextureGPU {
    pub fn new(ctx: &gpu::Context, texture: &Texture, image: &ImageGPU) -> Self {
        let view = image.image.create_view(&Default::default());
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(texture.label.as_str()),
            address_mode_u: match texture.sampler.wrap_u {
                WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
                WrappingMode::MirroredRepeat => AddressMode::MirrorRepeat,
                WrappingMode::Repeat => AddressMode::Repeat,
            },
            address_mode_v: match texture.sampler.wrap_v {
                WrappingMode::ClampToEdge => AddressMode::ClampToEdge,
                WrappingMode::MirroredRepeat => AddressMode::MirrorRepeat,
                WrappingMode::Repeat => AddressMode::Repeat,
            },
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: texture
                .sampler
                .mag_filter
                .map(|filter| match filter {
                    MinMagFilter::Nearest => FilterMode::Nearest,
                    MinMagFilter::Linear => FilterMode::Linear,
                })
                .unwrap_or_default(),
            min_filter: texture
                .sampler
                .min_filter
                .map(|filter| match filter {
                    MinMagFilter::Nearest => FilterMode::Nearest,
                    MinMagFilter::Linear => FilterMode::Linear,
                })
                .unwrap_or_default(),
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self { label: texture.label.clone(), view, sampler }
    }
}
