use image::ImageEncoder;
use pp_core::material::image::{Format, Image};

/// Converts a pp_core Image to GLTF Image with embedded base64 data
pub fn save_image(image: &Image) -> gltf_json::Image {
    use gltf_json::image;

    // Convert image to PNG format for embedding
    let uri = image_to_data_uri(image);
    image::Image {
        name: Some(image.label.clone()),
        uri: Some(uri),
        mime_type: Some(image::MimeType("image/png".to_string())),
        buffer_view: None,
        extensions: Default::default(),
        extras: Default::default(),
    }
}

pub fn load_image(gltf_image: &gltf::Image, buffers: &[gltf::buffer::Data], index: usize) -> Image {
    // Get the image data from the GLTF image source
    let image_data = match gltf_image.source() {
        gltf::image::Source::View { view, mime_type: _ } => {
            // Image is stored in a buffer view
            let buffer = &buffers[view.buffer().index()];
            let start = view.offset();
            let end = start + view.length();
            &buffer[start..end]
        }
        gltf::image::Source::Uri { uri, mime_type: _ } => {
            // For data URIs, parse the base64 data
            if let Some(base64_data) = uri
                .strip_prefix("data:image/png;base64,")
                .or_else(|| uri.strip_prefix("data:image/jpeg;base64,"))
            {
                // Decode base64 - we'll need to allocate here
                match base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    base64_data,
                ) {
                    Ok(decoded) => {
                        // We need a static reference, so we'll process immediately
                        return decode_image_data(&decoded, gltf_image.name(), index);
                    }
                    Err(_) => {
                        // Failed to decode, return placeholder
                        return create_placeholder_image(gltf_image.name(), index);
                    }
                }
            } else {
                // External file reference - not supported yet, return placeholder
                return create_placeholder_image(gltf_image.name(), index);
            }
        }
    };

    decode_image_data(image_data, gltf_image.name(), index)
}

fn decode_image_data(data: &[u8], name: Option<&str>, index: usize) -> Image {
    // Try to decode the image
    let img = match image::load_from_memory(data) {
        Ok(img) => img,
        Err(_) => return create_placeholder_image(name, index),
    };

    // Convert to RGBA8 for consistency
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    Image {
        label: name.map(|s| s.to_string()).unwrap_or_else(|| format!("Image{}", index)),
        pixels: rgba.into_raw(),
        width,
        height,
        format: Format::R8G8B8A8,
    }
}

fn create_placeholder_image(name: Option<&str>, index: usize) -> Image {
    // Create a 1x1 white pixel as placeholder
    Image {
        label: name.map(|s| s.to_string()).unwrap_or_else(|| format!("Image{}", index)),
        pixels: vec![255u8; 4],
        width: 1,
        height: 1,
        format: Format::R8G8B8A8,
    }
}

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
