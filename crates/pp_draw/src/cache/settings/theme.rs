use hex_color::HexColor;
use pp_editor::preferences::theme::{Theme, ThemeColors, ThemeSizes};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ThemeUniform {
    sizes: ThemeSizesUniform,
    colors: ThemeColorsUniform,
}

impl ThemeUniform {
    pub fn new(value: &Theme, overrides: &ThemeOverrides) -> Self {
        Self {
            sizes: ThemeSizesUniform::new(&value.sizes, overrides),
            colors: ThemeColorsUniform::new(&value.colors, overrides),
        }
    }

    pub fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            count: None,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }
    }
}

/// Per-pass adjustments to the user's theme.
///
/// The viewport renders with the defaults; the print pass overrides both, since
/// it rasterizes at a far higher pixel density than a screen and must not bake
/// transient editor state into the printout.
#[derive(Debug, Clone, Copy)]
pub struct ThemeOverrides {
    /// Multiplier on every size expressed in device pixels, so strokes keep a
    /// constant *physical* thickness as the pixel density changes. 1.0 is the
    /// ~96 DPI a CSS pixel assumes.
    pub stroke_scale: f32,
    /// Whether selected and active elements are highlighted at all.
    pub selection: bool,
    /// Replaces the colors that are otherwise pitch black - the default edge
    /// ink and the piece silhouette - with a softer shade. `None` leaves the
    /// user's theme colors alone.
    ///
    /// Annotated edges keep their own color: a cut line is red because that is
    /// what tells it apart from a fold, and washing it out would cost more than
    /// the softer look gains.
    pub ink: Option<HexColor>,
}

impl Default for ThemeOverrides {
    fn default() -> Self {
        Self { stroke_scale: 1.0, selection: true, ink: None }
    }
}

/// The `sizes` half of the theme uniform.
///
/// Every shader binding the theme declares this struct too, and `colors`
/// follows it, so a shader that declares fewer fields than are here reads the
/// colors from the wrong offset. Adding a field means updating *all* of the
/// `struct ThemeSizes` declarations under `engines/**/shaders/`, not only the
/// shaders that read the new field.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ThemeSizesUniform {
    line_width: f32,
    line_width_thick: f32,
    point_size: f32,
    /// Read as a boolean by `lines.wgsl`; every other shader ignores it.
    fold_lines: f32,
    /// [`ThemeOverrides::stroke_scale`]. The widths above are pre-scaled by it,
    /// so only shaders with hardcoded pixel lengths of their own (the fold
    /// dashes in `lines.wgsl`) need to read it.
    stroke_scale: f32,
    /// [`ThemeOverrides::selection`], as a boolean.
    selection: f32,
    /// Pads the struct out to the 16-byte alignment `ThemeColorsUniform` needs.
    _pad: [f32; 2],
}

impl ThemeSizesUniform {
    fn new(value: &ThemeSizes, overrides: &ThemeOverrides) -> Self {
        let k = overrides.stroke_scale;
        Self {
            line_width: value.line_width * k,
            line_width_thick: value.line_width_thick * k,
            point_size: value.point_size * k,
            fold_lines: if value.fold_lines { 1.0 } else { 0.0 },
            stroke_scale: k,
            selection: if overrides.selection { 1.0 } else { 0.0 },
            _pad: [0.0; 2],
        }
    }
}

const U8_MAX: f32 = u8::MAX as f32;
fn hex_color_to_f32(color: &HexColor) -> [f32; 4] {
    [
        color.r as f32 / U8_MAX,
        color.g as f32 / U8_MAX,
        color.b as f32 / U8_MAX,
        color.a as f32 / U8_MAX,
    ]
}

/// Automatically generates the implementation of the `ThemeColors` struct
/// and its default values in a much less verbose way.
macro_rules! define_theme_colors_gpu {
    ($($name:ident),* $(,)?) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct ThemeColorsUniform {
            $($name: [f32; 4],)*
        }

        impl ThemeColorsUniform {
            fn new(value: &ThemeColors, overrides: &ThemeOverrides) -> Self {
                let mut colors = Self {
                    $($name: hex_color_to_f32(&value.$name),)*
                };
                if let Some(ink) = overrides.ink {
                    let ink = hex_color_to_f32(&ink);
                    colors.ink = ink;
                    colors.edge_boundary = ink;
                }
                colors
            }
        }
    };
}

// This needs to have an even number of items for alignment reasons.
//
// The order has to match the `struct ThemeColors` every shader declares, since
// that is what decides which color lands in which slot - the names only line
// the fields up with [`ThemeColors`] on the Rust side.
define_theme_colors_gpu! {
    background,
    grid,
    grid_axis_x,
    grid_axis_y,
    element_active,
    element_selected,
    edge_cut,
    edge_boundary,
    ink,
    padding
}

