struct ThemeSizes { line_width: f32, line_width_thick: f32, point_size: f32 };
struct ThemeColors { 
  background: vec4<f32>,
  grid: vec4<f32>,
  grid_axis_x: vec4<f32>,
  grid_axis_y: vec4<f32>,
  element_active: vec4<f32>,
  element_selected: vec4<f32>,
  edge_cut: vec4<f32>,
  edge_boundary: vec4<f32>,
};
struct Theme { sizes: ThemeSizes, colors: ThemeColors };
@group(0) @binding(0) var<uniform> theme: Theme;
struct Viewport { position: vec2<f32>, dimensions: vec2<f32> };
struct Camera { view_proj: mat4x4<f32>, eye: vec4<f32> };
@group(1) @binding(0) var<uniform> viewport: Viewport;
@group(1) @binding(1) var<uniform> camera: Camera;

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

    var p = (in.offset * 2 - 1) * 8.0;
    out.world_position = vec3<f32>(p, 0.0);
    out.clip_position = camera.view_proj * vec4<f32>(out.world_position, 1.0);
    return out;
}

// Fragment shader
fn grid(pos: vec3<f32>, scale: f32) -> vec4<f32> {
    let fade_radius = 5.0;
    var distance = length(pos.xy);
    var fade = smoothstep(fade_radius, 0.0, distance);
    // Scale the world-space position for the grid
    let coord = pos.xy * scale;
    // Compute screen-space derivatives for consistent line thickness
    let derivative = fwidth(coord);

    // Create grid lines by isolating fractional parts
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;

    // Determine line intensity with antialiasing
    let line = min(grid.x, grid.y);

    // Base grid color
    var axis_color = theme.colors.grid.xyz;

    // Highlight axes
    if abs(coord.y) < 0.05 {
        axis_color = theme.colors.grid_axis_x.xyz;
    } else if abs(coord.x) < 0.05 {
        axis_color = theme.colors.grid_axis_y.xyz;
    }

    return vec4<f32>(axis_color, fade - min(line, fade));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return grid(in.world_position, 2.0);
}
