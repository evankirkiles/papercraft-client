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

pub fn load_image(image: &gltf::Image, index: usize) -> Image {
    // For now, create a placeholder - full implementation would decode the data URI
    // TODO: Decode base64 data URI and parse PNG
    Image {
        label: image.name().map(|e| e.to_string()).unwrap_or_else(|| format!("Image{}", index)),
        pixels: vec![255u8; 4], // Placeholder: 1x1 white pixel
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
