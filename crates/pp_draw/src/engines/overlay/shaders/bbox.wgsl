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
};
struct Theme { sizes: ThemeSizes, colors: ThemeColors };
@group(0) @binding(0) var<uniform> theme: Theme;
struct Viewport { position: vec2<f32>, dimensions: vec2<f32> };
struct Camera { view_proj: mat4x4<f32>, eye: vec4<f32> };
@group(1) @binding(0) var<uniform> viewport: Viewport;
@group(1) @binding(1) var<uniform> camera: Camera;

struct VertexInput {
    @location(0) offset: vec2<f32>,
    @location(1) v0_pos: vec3<f32>,
    @location(2) v1_pos: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Screen-space expansion of a world-space line segment into a constant-
    // pixel-width quad, matching engines::ink::lines's `_vs_clip_pos`
    // (simplified: no piece transform, since bbox edges are already world-space).
    var clip_v0 = camera.view_proj * vec4<f32>(in.v0_pos, 1.0);
    var clip_v1 = camera.view_proj * vec4<f32>(in.v1_pos, 1.0);
    var screen_v0 = viewport.dimensions * (0.5 * clip_v0.xy / clip_v0.w + 0.5);
    var screen_v1 = viewport.dimensions * (0.5 * clip_v1.xy / clip_v1.w + 0.5);

    var basis_x = screen_v1 - screen_v0;
    var basis_y = normalize(vec2<f32>(-basis_x.y, basis_x.x));
    var pt = screen_v0 + in.offset.x * basis_x + (0.5 - in.offset.y) * basis_y * theme.sizes.line_width;
    var clip = mix(clip_v0, clip_v1, in.offset.x);
    out.clip_position = vec4<f32>(clip.w * (2.0 * pt / viewport.dimensions - 1.0), clip.z, clip.w);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return theme.colors.grid;
}
