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
    @location(1) v0_pos: vec3<f32>,
    @location(2) v1_pos: vec3<f32>,
    @location(3) top0_pos: vec3<f32>,
    @location(4) top1_pos: vec3<f32>,
    @location(5) flap_flags: u32,
    @location(6) flags: u32,
    @location(7) select_idx: vec4<u32>,
    @builtin(vertex_index) vertex_index: u32
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) select_idx: vec4<u32>
};

// Edge flags
const E_FLAG_SELECTED: u32 = (u32(1) << 0);
const E_FLAG_ACTIVE: u32 = (u32(1) << 1);
const E_FLAG_V0_SELECTED: u32 = (u32(1) << 2);
const E_FLAG_V1_SELECTED: u32 = (u32(1) << 3);
const E_FLAG_CUT: u32 = (u32(1) << 4);

// Flap flags
const F_FLAG_EXISTS: u32 = (u32(1) << 0);

// The 4 corners of the trapezoid flap: bottom-left, bottom-right, top-right,
// top-left.
//
// The shape is decided on the CPU by `pp_core::mesh::flap::flap_corners` — the
// bottom pair is the base edge this flap folds along, the top pair arrives in
// its own buffer. Keep it that way: the vector print path strokes these same
// four points, so a copy of the trapezoid math living here too would let the
// printed tab drift from the one on screen.
fn _flap_corners(in: VertexInput) -> array<vec3<f32>, 4> {
    return array<vec3<f32>, 4>(in.v0_pos, in.v1_pos, in.top1_pos, in.top0_pos);
}

// Calculates the colors of flaps as would be seen on-screen.
fn _vs_color(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    // Color the flap (each vertex) based on its select status. Nonexistent
    // flaps should be clipped out already, but just in case...
    if (theme.sizes.selection > 0.0 && bool(in.flags & E_FLAG_SELECTED)) {
      out.color = mix(out.color, theme.colors.element_selected, 0.5);
    }

    // Add the edge index for the selection engine
    out.select_idx = in.select_idx;
    return out;
}

// Calculates the colors of edges as would be seen on-screen.
fn _vs_color_edge(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // Color the flap (each vertex) based on its select status
    if (theme.sizes.selection > 0.0 && bool(in.flags & E_FLAG_SELECTED)) {
      out.color = theme.colors.element_selected;
    }

    // Add the edge index for the selection engine
    out.select_idx = in.select_idx;
    return out;
}

// Calculates the clip position of edge vertices based on the width of the line
fn _vs_clip_pos(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    
    // Compute corners of the flap trapezoid
    let corners = _flap_corners(in);
  
    // Interpolate between corners of the flap based on input verts
    let base_pos = mix(corners[0], corners[1], in.offset.x);
    let top_pos = mix(corners[3], corners[2], in.offset.x);
    let pos = mix(base_pos, top_pos, in.offset.y);
    out.clip_position =
        _apply_depth_offset(camera.view_proj * piece.affine * vec4<f32>(pos, 1.0));

    // If flap doesn't exist, push it offscreen to avoid rasterization
    if (bool(in.flap_flags ^ F_FLAG_EXISTS)) {
        out.clip_position.z = -100.0;
    }

    return out;
}

// Calculates the clip position of edge vertices based on the width of the line
fn _vs_clip_pos_edge(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;

    // Side 0 of the trapezoid is its base, which lies right on the piece edge
    // the flap hangs off of. That seam is a fold rather than a cut, so leave it
    // to the edge line pipeline and only outline the three free sides.
    let side = u32(in.vertex_index / 6);
    if (side == 0u || !bool(in.flap_flags & F_FLAG_EXISTS)) {
        out.clip_position.z = -100.0;
        return out;
    }

    // Get the corners of the flap trapezoid
    let corners = _flap_corners(in);

    // Get the current vertex and the next vertex
    let p0 = corners[side];
    let p1 = corners[(side + 1u) % 4u];

    // Find screen-space positions of each vertex
    var clip_v0 = camera.view_proj * piece.affine * vec4<f32>(p0, 1.0);
    var clip_v1 = camera.view_proj * piece.affine * vec4<f32>(p1, 1.0);
    var screen_v0 = viewport.dimensions * (0.5 * clip_v0.xy / clip_v0.w + 0.5);
    var screen_v1 = viewport.dimensions * (0.5 * clip_v1.xy / clip_v1.w + 0.5);

    // Expand into line segment
    var basis_x = screen_v1 - screen_v0;
    var basis_y = normalize(vec2<f32>(-basis_x.y, basis_x.x));
    var pt = screen_v0 + in.offset.x * basis_x + (0.5 - in.offset.y) * basis_y * theme.sizes.line_width;
    var clip = mix(clip_v0, clip_v1, in.offset.x);
    out.clip_position = _apply_depth_offset(
        vec4<f32>(clip.w * (2.0 * pt / viewport.dimensions - 1.0), clip.z, clip.w));

    return out;
}

// [VS.1] Flap
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out = _vs_color(in, out);
    out = _vs_clip_pos(in, out);
    return out;
}

// [VS.2] Flap Edges
@vertex
fn vs_edge(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out = _vs_color_edge(in, out);
    out = _vs_clip_pos_edge(in, out);
    return out;
}

// [FS.1] Normal rendering
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// [FS.2] Select index rendering
@fragment
fn fs_select(in: VertexOutput) -> @location(0) vec4<u32> {
    return in.select_idx;
}
