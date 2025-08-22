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
struct ToolRotate { center_pos: vec2<f32>, curr_pos: vec2<f32> };
@group(2) @binding(0) var<uniform> tool: ToolRotate;

struct VertexInput {
    @location(0) offset: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) line_uv: f32,          // 0 to 1 along the line
};

// How thick the line actually is
const LINE_WIDTH_THIN: f32 = 1.5;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let a = tool.center_pos;
    let b = tool.curr_pos;

    let dir = normalize(b - a);
    let perp = vec2<f32>(-dir.y, dir.x);

    // Compute vertex position in screen-space
    let along = mix(a, b, in.offset.x);
    let offset = (in.offset.y - 0.5) * LINE_WIDTH_THIN * perp; // shift ±0.5px
    let pos = along + offset;
    out.line_uv = in.offset.x;

    // Convert to NDC
    let rel = (pos - viewport.position) / viewport.dimensions;
    let ndc = rel * 2.0 - 1.0;
    out.clip_position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return vec4<f32>(1.0, 0.0, 0.0, 1.0); // white dot
    let dash_length = 5.0; // pixels
    let total_length = distance(tool.center_pos, tool.curr_pos);
    let dist_along_line = in.line_uv * total_length;
    let show = floor(dist_along_line / dash_length) % 2.0 == 0.0;
    if (show) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0); // black line
    } else {
        return vec4<f32>(1.0, 1.0, 1.0, 0.0); // clear gaps
    }
}
