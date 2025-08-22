use hex_color::HexColor;
use pp_editor::settings::theme::{Theme, ThemeColors, ThemeSizes};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ThemeUniform {
    sizes: ThemeSizesUniform,
    colors: ThemeColorsUniform,
}

impl ThemeUniform {
    pub fn from(value: &Theme) -> Self {
        Self { sizes: (&value.sizes).into(), colors: (&value.colors).into() }
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ThemeSizesUniform {
    line_width: f32,
    line_width_thick: f32,
    point_size: f32,
    padding: f32,
}

impl From<&ThemeSizes> for ThemeSizesUniform {
    fn from(value: &ThemeSizes) -> Self {
        Self {
            line_width: value.line_width,
            line_width_thick: value.line_width_thick,
            point_size: value.point_size,
            padding: 0.0,
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

        impl From<&ThemeColors> for ThemeColorsUniform {
            fn from(value: &ThemeColors) -> Self {
                Self {
                    $($name: hex_color_to_f32(&value.$name),)*
                }
            }
        }
    };
}

// This needs to have an even number of items for alignment reasons
define_theme_colors_gpu! {
    background,
    grid,
    grid_axis_x,
    grid_axis_y,
    element_active,
    element_selected,
    edge_cut,
    padding
}
