struct ThemeSizes {
  line_width: f32,
  line_width_thick: f32,
  point_size: f32,
  fold_lines: f32,
  // Scales lengths this shader hardcodes in pixels, so they keep a
  // constant physical size as the pixel density changes.
  stroke_scale: f32,
  // Whether selected / active elements are highlighted at all. Off for
  // print, which must not bake transient editor state into the page.
  selection: f32,
};
struct ThemeColors { 
  background: vec4<f32>,
  grid: vec4<f32>,
  grid_axis_x: vec4<f32>,
  grid_axis_y: vec4<f32>,
  element_active: vec4<f32>,
  element_selected: vec4<f32>,
  edge_cut: vec4<f32>,
  edge_boundary: vec4<f32>,
  // The default stroke color for edges carrying no annotation of their own.
  ink: vec4<f32>,
  padding: vec4<f32>,
};
struct Theme { sizes: ThemeSizes, colors: ThemeColors };
@group(0) @binding(0) var<uniform> theme: Theme;
struct Viewport { position: vec2<f32>, dimensions: vec2<f32> };
struct Camera { view_proj: mat4x4<f32>, eye: vec4<f32> };
@group(1) @binding(0) var<uniform> viewport: Viewport;
@group(1) @binding(1) var<uniform> camera: Camera;
struct PageLayout {
  margin_start: vec2<f32>,
  margin_end: vec2<f32>,
  dimensions: vec2<f32>,
  grid_dimensions: vec2<f32>,
};
@group(2) @binding(0) var<uniform> page: PageLayout;


struct VertexInput {
   @location(0) offset: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) world_position: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Pad the grid out past the outer edges of the whole page grid. Has to
    // leave room for the longest fade the fragment stage draws, AXIS_FADE_RADIUS.
    let PAD = 5.0;
    let DIMS = page.dimensions * page.grid_dimensions;

    var p = ((in.offset * (DIMS + PAD * 2)) - PAD) * vec2<f32>(1.0, -1.0);
    out.world_position = vec3<f32>(p, 0.0);
    out.clip_position = camera.view_proj * vec4<f32>(out.world_position, 1.0);
    return out;
}

// How far past the edges of the page grid the grid lines keep going before
// they fade out, and the longer run the axes get. Both have to stay inside the
// PAD the vertex stage builds the quad with.
const GRID_FADE_RADIUS: f32 = 1.5;
const AXIS_FADE_RADIUS: f32 = 4.0;

// Fragment shader
fn grid(pos: vec3<f32>, scale: f32) -> vec4<f32> {
    let width = page.dimensions.x * page.grid_dimensions.x;
    let height = page.dimensions.y * page.grid_dimensions.y;
    // Scale the world-space position for the grid
    let coord = pos.xy * scale;
    // Compute screen-space derivatives for consistent line thickness
    let derivative = fwidth(coord);

    // Create grid lines by isolating fractional parts
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;

    // Determine line intensity with antialiasing
    let line = min(grid.x, grid.y);

    // Base grid color
    var axis_color = vec3<f32>(0.1, 0.1, 0.1);

    // Highlight axes
    let is_x_axis = abs(coord.y) < 0.05;
    let is_y_axis = abs(coord.x) < 0.05;
    if is_x_axis {
        axis_color = theme.colors.grid_axis_x.xyz;
    } else if is_y_axis {
        axis_color = theme.colors.grid_axis_y.xyz;
    // Highlight bounds (red for Y=0 (X axis), green for X=0 (Y axis))
    } else if abs(coord.y + height * scale) < 0.05 || abs(coord.x - width * scale) < 0.05 {
        axis_color = theme.colors.grid.xyz;
    }

    // Fade the ink out past the edges of the pages. The axes get a longer run
    // than the grid lines, but only outward into the printable quadrant, so
    // they read as the quadrant's edges rather than as a cross through it.
    let dist = length(pos.xy - clamp(pos.xy, vec2<f32>(0, -1 * height), vec2<f32>(width, 0)));
    var fade_radius = GRID_FADE_RADIUS;
    if (is_x_axis && pos.x > width) || (is_y_axis && pos.y < -1 * height) {
        fade_radius = AXIS_FADE_RADIUS;
    }
    let fade = smoothstep(fade_radius, 0.0, dist);
    return vec4<f32>(axis_color, fade - min(line, fade));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1 world unit = 1 cm, so a scale of 1.0 spaces grid lines every centimeter.
    return grid(in.world_position, 1.0);
}
