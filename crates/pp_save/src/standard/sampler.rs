use pp_core::material::texture::{MinMagFilter, Sampler, WrappingMode};

/// Converts a pp_core Sampler to GLTF Sampler
pub fn save_sampler(sampler: &Sampler) -> gltf_json::texture::Sampler {
    use gltf_json::texture;
    use gltf_json::validation::Checked::Valid;
    texture::Sampler {
        name: None,
        extensions: Default::default(),
        extras: Default::default(),
        mag_filter: sampler.mag_filter.map(|f| {
            Valid(match f {
                MinMagFilter::Nearest => texture::MagFilter::Nearest,
                MinMagFilter::Linear => texture::MagFilter::Linear,
            })
        }),
        min_filter: sampler.min_filter.map(|f| {
            Valid(match f {
                MinMagFilter::Nearest => texture::MinFilter::Nearest,
                MinMagFilter::Linear => texture::MinFilter::Linear,
            })
        }),
        wrap_s: Valid(match sampler.wrap_u {
            WrappingMode::ClampToEdge => texture::WrappingMode::ClampToEdge,
            WrappingMode::MirroredRepeat => texture::WrappingMode::MirroredRepeat,
            WrappingMode::Repeat => texture::WrappingMode::Repeat,
        }),
        wrap_t: Valid(match sampler.wrap_v {
            WrappingMode::ClampToEdge => texture::WrappingMode::ClampToEdge,
            WrappingMode::MirroredRepeat => texture::WrappingMode::MirroredRepeat,
            WrappingMode::Repeat => texture::WrappingMode::Repeat,
        }),
    }
}

pub fn load_sampler(gltf_sampler: &gltf::texture::Sampler) -> Sampler {
    use gltf::texture;
    Sampler {
        min_filter: gltf_sampler.min_filter().map(|f| match f {
            texture::MinFilter::Nearest
            | texture::MinFilter::NearestMipmapNearest
            | texture::MinFilter::NearestMipmapLinear => MinMagFilter::Nearest,
            texture::MinFilter::Linear
            | texture::MinFilter::LinearMipmapNearest
            | texture::MinFilter::LinearMipmapLinear => MinMagFilter::Linear,
        }),
        mag_filter: gltf_sampler.mag_filter().map(|f| match f {
            texture::MagFilter::Nearest => MinMagFilter::Nearest,
            texture::MagFilter::Linear => MinMagFilter::Linear,
        }),
        wrap_u: match &gltf_sampler.wrap_s() {
            texture::WrappingMode::ClampToEdge => WrappingMode::ClampToEdge,
            texture::WrappingMode::MirroredRepeat => WrappingMode::MirroredRepeat,
            texture::WrappingMode::Repeat => WrappingMode::Repeat,
        },
        wrap_v: match &gltf_sampler.wrap_t() {
            texture::WrappingMode::ClampToEdge => WrappingMode::ClampToEdge,
            texture::WrappingMode::MirroredRepeat => WrappingMode::MirroredRepeat,
            texture::WrappingMode::Repeat => WrappingMode::Repeat,
        },
    }
}
