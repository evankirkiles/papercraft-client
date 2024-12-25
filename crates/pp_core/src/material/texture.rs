use image::GenericImageView;

pub struct Texture<'a> {
    label: Option<&'a str>,
    image: image::DynamicImage,
}
