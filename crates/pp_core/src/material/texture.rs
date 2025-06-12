use crate::id;

#[derive(Debug)]
pub struct Texture {
    pub label: String,
    pub image: id::ImageId,
    pub sampler: Sampler,
}

/// Magnification filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum MinMagFilter {
    #[default]
    /// Corresponds to `GL_NEAREST`.
    Nearest = 1,
    /// Corresponds to `GL_LINEAR`.
    Linear,
}

/// Texture co-ordinate wrapping mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum WrappingMode {
    /// Corresponds to `GL_CLAMP_TO_EDGE`.
    ClampToEdge = 1,
    /// Corresponds to `GL_MIRRORED_REPEAT`.
    MirroredRepeat,
    #[default]
    /// Corresponds to `GL_REPEAT`.
    Repeat,
}

#[derive(Debug, Default)]
pub struct Sampler {
    pub wrap_u: WrappingMode,
    pub wrap_v: WrappingMode,
    pub min_filter: Option<MinMagFilter>,
    pub mag_filter: Option<MinMagFilter>,
}
