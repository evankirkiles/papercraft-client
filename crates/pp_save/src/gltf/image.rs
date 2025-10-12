use gltf_json as json;
use gltf_json::validation::Checked::Valid;
use image::ImageEncoder;
use pp_core::material::image::{Format, Image};
use pp_core::material::texture::{MinMagFilter, Sampler, WrappingMode};

/// Converts a pp_core Image to GLTF Image with embedded base64 data
pub fn save_image(image: &Image) -> json::Image {
    // Convert image to PNG format for embedding
    let uri = image_to_data_uri(image);

    json::Image {
        name: Some(image.label.clone()),
        uri: Some(uri),
        mime_type: Some(json::image::MimeType("image/png".to_string())),
        buffer_view: None,
        extensions: Default::default(),
        extras: Default::default(),
    }
}

// pub fn load_image(image: json::Image) -> Image {
//     // Map three-channel pixel data into four-channel pixel data (Alpha being 1)
//     let pixels = match image.format {
//         gltf::image::Format::R8G8B8 => {
//             // Convert u8 RGB to u8 RGBA (alpha = 255)
//             image
//                 .pixels
//                 .chunks(3)
//                 .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
//                 .collect::<Vec<u8>>()
//         }
//         gltf::image::Format::R16G16B16 => {
//             // Convert u16 RGB to u16 RGBA (alpha = u16::MAX)
//             let mut rgba = Vec::with_capacity(image.pixels.len() / 3 * 4);
//             for chunk in image.pixels.chunks_exact(6) {
//                 let r = u16::from_le_bytes([chunk[0], chunk[1]]);
//                 let g = u16::from_le_bytes([chunk[2], chunk[3]]);
//                 let b = u16::from_le_bytes([chunk[4], chunk[5]]);
//                 rgba.extend_from_slice(&r.to_le_bytes());
//                 rgba.extend_from_slice(&g.to_le_bytes());
//                 rgba.extend_from_slice(&b.to_le_bytes());
//                 rgba.extend_from_slice(&u16::MAX.to_le_bytes());
//             }
//             rgba
//         }
//         gltf::image::Format::R32G32B32FLOAT => {
//             // Convert f32 RGB to f32 RGBA (alpha = 1.0)
//             let mut rgba = Vec::with_capacity(image.pixels.len() / 3 * 4);
//             for chunk in image.pixels.chunks_exact(12) {
//                 let r = f32::from_le_bytes(chunk[0..4].try_into().unwrap());
//                 let g = f32::from_le_bytes(chunk[4..8].try_into().unwrap());
//                 let b = f32::from_le_bytes(chunk[8..12].try_into().unwrap());
//                 rgba.extend_from_slice(&r.to_le_bytes());
//                 rgba.extend_from_slice(&g.to_le_bytes());
//                 rgba.extend_from_slice(&b.to_le_bytes());
//                 rgba.extend_from_slice(&1.0f32.to_le_bytes());
//             }
//             rgba
//         }
//         _ => image.pixels.clone(),
//     };
//     Image {
//         label: format!("Image{i:?}").as_str().into(),
//         pixels,
//         width: image.width,
//         height: image.height,
//         format: match image.format {
//             gltf::image::Format::R8 => Format::R8,
//             gltf::image::Format::R8G8 => Format::R8G8,
//             gltf::image::Format::R8G8B8A8 => Format::R8G8B8A8,
//             gltf::image::Format::R16 => Format::R16,
//             gltf::image::Format::R16G16 => Format::R16G16,
//             gltf::image::Format::R16G16B16A16 => Format::R16G16B16A16,
//             gltf::image::Format::R32G32B32A32FLOAT => Format::R32G32B32A32FLOAT,
//             // Three-channel textures are given an alpha channel so they can
//             // be handled by `wgpu` (which doesn't support 3-channel textures)
//             gltf::image::Format::R8G8B8 => Format::R8G8B8A8,
//             gltf::image::Format::R16G16B16 => Format::R16G16B16A16,
//             gltf::image::Format::R32G32B32FLOAT => Format::R32G32B32A32FLOAT,
//         },
//     }
// }

/// Converts image data to a data URI
fn image_to_data_uri(image: &Image) -> String {
    use std::io::Cursor;

    let mut buffer = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(Cursor::new(&mut buffer));

    // Convert our format to image crate's ColorType
    let color_type = match image.format {
        Format::R8 => image::ExtendedColorType::L8,
        Format::R8G8 => image::ExtendedColorType::La8,
        Format::R8G8B8 => image::ExtendedColorType::Rgb8,
        Format::R8G8B8A8 => image::ExtendedColorType::Rgba8,
        Format::R16 => image::ExtendedColorType::L16,
        Format::R16G16 => image::ExtendedColorType::La16,
        Format::R16G16B16 => image::ExtendedColorType::Rgb16,
        Format::R16G16B16A16 => image::ExtendedColorType::Rgba16,
        // For float formats, we'll convert to 8-bit (this is a simplification)
        Format::R32G32B32FLOAT => image::ExtendedColorType::Rgb8,
        Format::R32G32B32A32FLOAT => image::ExtendedColorType::Rgba8,
    };

    encoder
        .write_image(&image.pixels, image.width, image.height, color_type)
        .expect("Failed to encode image");

    format!(
        "data:image/png;base64,{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &buffer)
    )
}

/// Converts a pp_core Sampler to GLTF Sampler
pub fn save_sampler(sampler: &Sampler) -> json::texture::Sampler {
    json::texture::Sampler {
        mag_filter: sampler.mag_filter.map(|f| Valid(minmag_to_gltf_mag(f))),
        min_filter: sampler.min_filter.map(|f| Valid(minmag_to_gltf_min(f))),
        wrap_s: Valid(wrapping_to_gltf(sampler.wrap_u)),
        wrap_t: Valid(wrapping_to_gltf(sampler.wrap_v)),
        name: None,
        extensions: Default::default(),
        extras: Default::default(),
    }
}

fn minmag_to_gltf_min(filter: MinMagFilter) -> json::texture::MinFilter {
    match filter {
        MinMagFilter::Nearest => json::texture::MinFilter::Nearest,
        MinMagFilter::Linear => json::texture::MinFilter::Linear,
    }
}

fn minmag_to_gltf_mag(filter: MinMagFilter) -> json::texture::MagFilter {
    match filter {
        MinMagFilter::Nearest => json::texture::MagFilter::Nearest,
        MinMagFilter::Linear => json::texture::MagFilter::Linear,
    }
}

fn wrapping_to_gltf(mode: WrappingMode) -> json::texture::WrappingMode {
    match mode {
        WrappingMode::ClampToEdge => json::texture::WrappingMode::ClampToEdge,
        WrappingMode::MirroredRepeat => json::texture::WrappingMode::MirroredRepeat,
        WrappingMode::Repeat => json::texture::WrappingMode::Repeat,
    }
}
