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
struct PageLayout { 
  margin_start: vec2<f32>, 
  margin_end: vec2<f32>, 
  dimensions: vec2<f32>,
  padding: vec2<f32>,
};
@group(2) @binding(0) var<uniform> page: PageLayout;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec2<f32>,
    @location(1) offset: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) frag_uv: vec2<f32>,
    @location(1) local_pos: vec2<f32>
};

const LINE_WIDTH: f32 = 1.5;
const FLIP_Y: vec2<f32> = vec2<f32>(1.0, -1.0);

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = vec4<f32>((in.position + in.offset) * FLIP_Y * page.dimensions, 0.0, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    return out;
}

@vertex
fn vs_margins(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
 
    // Get the corners of the inner margin rect
    let p_0 = (in.position * page.dimensions + page.margin_start) * FLIP_Y;
    let p_1 = ((in.position + vec2<f32>(1.0, 1.0)) * page.dimensions - page.margin_end) * FLIP_Y;
    let corners = array<vec2<f32>, 4>(
      mix(p_0, p_1, vec2<f32>(0, 0)), // top-left
      mix(p_0, p_1, vec2<f32>(0, 1)), // top-right
      mix(p_0, p_1, vec2<f32>(1, 1)), // bottom-right
      mix(p_0, p_1, vec2<f32>(1, 0)), // bottom-left
    ); 

    // Get the current vertex and the next vertex
    let p0 = vec3<f32>(corners[u32(in.vertex_index / 6)], 0);
    let p1 = vec3<f32>(corners[(u32(in.vertex_index / 6) + 1) % 4], 0);
    let world_p0 = p0 + normalize(camera.eye.xyz - p0) * camera.eye.w * 0.001;
    let world_p1 = p1 + normalize(camera.eye.xyz - p1) * camera.eye.w * 0.001;

    // Find screen-space positions of each vertex
    var clip_v0 = camera.view_proj * vec4<f32>(world_p0, 1.0);
    var clip_v1 = camera.view_proj * vec4<f32>(world_p1, 1.0);
    var screen_v0 = viewport.dimensions * (0.5 * clip_v0.xy / clip_v0.w + 0.5);
    var screen_v1 = viewport.dimensions * (0.5 * clip_v1.xy / clip_v1.w + 0.5);

    // Expand into line segment
    var basis_x = screen_v1 - screen_v0;
    var basis_y = normalize(vec2<f32>(-basis_x.y, basis_x.x));
    var pt = screen_v0 + in.offset.x * basis_x + (0.5 - in.offset.y) * basis_y * LINE_WIDTH;
    var clip = mix(clip_v0, clip_v1, in.offset.x);
    out.clip_position = vec4<f32>(clip.w * (2.0 * pt / viewport.dimensions - 1.0), clip.z, clip.w);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.05, 0.05, 0.05, 0.9);
}

@fragment
fn fs_margins(in: VertexOutput) -> @location(0) vec4<f32> {
    // TODO: Dotted here, too?
    return vec4<f32>(0.5, 0.5, 0.5, 0.5);
}
