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
struct Piece { affine: mat4x4<f32>, depth_slot: f32 };
@group(2) @binding(0) var<uniform> piece: Piece;

// Where this pipeline's geometry sits in the stack of coplanar geometry, as a
// `DepthClass` discriminant. Set per-pipeline; see `engines::ink::DepthClass`.
override depth_class: f32 = 0.0;

// How far one class lifts geometry toward the eye, as a fraction of that
// geometry's own distance from the camera. Being *relative* is the point: it
// holds at any zoom and on any model scale.
const DEPTH_CLASS_STEP: f32 = 1.0 / 4096.0;

// How much of a class step a piece's slot may use. Well under 1, so a slot only
// ever breaks ties inside its own class and can't promote a piece into the next.
const DEPTH_SLOT_SPAN: f32 = 0.5;

// Lifts a projected position toward the viewer by its class, so that coplanar
// geometry resolves by what it *is* rather than by draw order or by whichever
// polygon happened to win the depth test.
//
// The lift is a fixed fraction of view depth, not a fixed amount of NDC depth.
// NDC depth goes as ~1/z, so with this projection (near 0.1, far 100+) the whole
// model lands in the top few percent of the depth range: a constant NDC offset
// that looks tiny is in fact a large part of the model's depth extent, and it
// stays constant as the camera dollies out while that extent keeps shrinking —
// so far-side geometry punches through. `1 - ndc_z` is proportional to
// `near / z_view`, so scaling by it turns the offset back into a constant
// relative step, small against real depth differences at every distance.
fn _apply_depth_offset(clip: vec4<f32>) -> vec4<f32> {
    let ndc_z = clip.z / clip.w;
    let units = depth_class + piece.depth_slot * DEPTH_SLOT_SPAN;
    let offset = units * DEPTH_CLASS_STEP * max(1.0 - ndc_z, 0.0);
    return vec4<f32>(clip.xy, (ndc_z - offset) * clip.w, clip.w);
}

struct VertexInput { 
    @location(0) offset: vec2<f32>,
    @location(1) pos: vec3<f32>,
    @location(2) flags: u32,
    @location(3) select_idx: vec4<u32>
};

struct VertexOutput { 
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) @interpolate(flat) select_idx: vec4<u32>
};

// Vertex flags
const FLAG_SELECTED: u32 = (u32(1) << 0);
const FLAG_ACTIVE: u32 = (u32(1) << 1);

// Calculates the colors of vertices as would be seen on-screen.
fn _vs_color(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // Color each vertex based on its select status
    if (bool(in.flags & FLAG_ACTIVE)) {
      out.color = theme.colors.element_active;
    } else if (bool(in.flags & FLAG_SELECTED)) {
      out.color = theme.colors.element_selected;
    }

    // Forward through selection index
    out.select_idx = in.select_idx;
    return out;
}

// Calculates the clip position of edge vertices, optionally with an affine 
// transformation (e.g. to use for pieces).
fn _vs_clip_pos(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    var clip_center = camera.view_proj * piece.affine * vec4<f32>(in.pos, 1.0);
    var ndc_offset = theme.sizes.point_size * (0.5 - in.offset) / viewport.dimensions;
    out.clip_position =
        _apply_depth_offset(clip_center + vec4<f32>(ndc_offset * clip_center.w, 0.0, 0.0));
    return out;
}

// [VS.1] Full mesh vertices
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out = _vs_color(in, out);
    out = _vs_clip_pos(in, out);
    return out;
}

// [FS.1] Normal rendering
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// [FS.2] X-Ray rendering
@fragment
fn fs_xray(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color * vec4<f32>(1.0, 1.0, 1.0, 0.05);
}

// [FS.3] Select index rendering
@fragment
fn fs_select(in: VertexOutput) -> @location(0) vec4<u32> {
    return in.select_idx;
}
