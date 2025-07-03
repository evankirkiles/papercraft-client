use pp_core::material::image::{Format, Image};
use wgpu::{util::DeviceExt, TextureDescriptor};

use crate::gpu;

#[derive(Clone, Debug)]
pub struct ImageGPU {
    image: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl ImageGPU {
    pub fn new(ctx: &gpu::Context, img: &Image) -> Self {
        let image = ctx.device.create_texture_with_data(
            &ctx.queue,
            &TextureDescriptor {
                label: Some(img.label.as_str()),
                mip_level_count: 1,
                sample_count: 1,
                format: match img.format {
                    Format::R8 => wgpu::TextureFormat::R8Unorm,
                    Format::R8G8 => wgpu::TextureFormat::Rg8Unorm,
                    Format::R8G8B8A8 => wgpu::TextureFormat::Rgba8Unorm,
                    Format::R16 => wgpu::TextureFormat::R16Unorm,
                    Format::R16G16 => wgpu::TextureFormat::Rg16Unorm,
                    Format::R16G16B16A16 => wgpu::TextureFormat::Rgba16Unorm,
                    Format::R32G32B32A32FLOAT => wgpu::TextureFormat::Rgba32Float,
                    Format::R8G8B8 => todo!(),
                    Format::R16G16B16 => todo!(),
                    Format::R32G32B32FLOAT => todo!(),
                },
                size: wgpu::Extent3d {
                    width: img.width,
                    height: img.height,
                    depth_or_array_layers: 1,
                },
                dimension: wgpu::TextureDimension::D2,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::wgt::TextureDataOrder::LayerMajor,
            &img.pixels,
        );
        Self { view: image.create_view(&Default::default()), image }
    }
}
