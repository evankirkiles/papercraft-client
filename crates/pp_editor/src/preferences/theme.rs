use hex_color::HexColor;
use serde::Serialize;
use tsify::Tsify;

/// User-provided visual settings
#[derive(Debug, Default, Tsify, Serialize)]
pub struct Theme {
    pub sizes: ThemeSizes,
    pub colors: ThemeColors,
}

#[derive(Debug, Copy, Clone, Tsify, Serialize)]
pub struct ThemeSizes {
    pub line_width: f32,
    pub line_width_thick: f32,
    pub point_size: f32,
    /// Whether fold (mountain / valley) lines are drawn at all. When off, the
    /// only strokes left are the piece silhouette — the mesh boundary and cut
    /// edges with no flap on that side. Edges a flap folds along stay blank,
    /// same as any other fold.
    ///
    /// Read by both the cutting viewport and the vector print path, so that what
    /// you see is what gets printed.
    pub fold_lines: bool,
}

impl Default for ThemeSizes {
    fn default() -> Self {
        Self { line_width: 1.5, line_width_thick: 4.0, point_size: 14.0, fold_lines: true }
    }
}

/// Automatically generates the implementation of the `ThemeColors` struct
/// and its default values in a much less verbose way.
macro_rules! define_theme_colors {
    ($($name:ident: $default:expr),* $(,)?) => {
        #[derive(Debug, Copy, Clone, Tsify, Serialize)]
        pub struct ThemeColors {
            $(pub $name: HexColor,)*
        }

        impl Default for ThemeColors {
            fn default() -> Self {
                Self {
                    $($name: HexColor::parse($default).unwrap(),)*
                }
            }
        }
    };
}

define_theme_colors! {
    background: "#000000",
    grid: "#303030",
    grid_axis_x: "#ff0000",
    grid_axis_y: "#00ff00",
    element_active: "#ffffff",
    element_selected: "#ffa500",
    edge_cut: "#ff0000",
    edge_boundary: "#000000",
    ink: "#000000",
    padding: "#000000"
}
