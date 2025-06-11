use image::{Pixel, Rgb};

#[derive(Debug)]
pub struct Texture {
    pub label: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<<Rgb<f32> as Pixel>::Subpixel>,
}
