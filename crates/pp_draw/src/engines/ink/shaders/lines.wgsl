struct ThemeSizes { line_width: f32, line_width_thick: f32, point_size: f32, fold_lines: f32 };
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
    @location(3) flags: u32,
    @location(4) select_idx: vec4<u32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // Screen-space distance along the segment from v0, in pixels. Interpolated
    // linearly so the dash phase doesn't get perspective-warped.
    @location(0) @interpolate(linear) dash_t: f32,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) select_idx: vec4<u32>,
    @location(3) @interpolate(flat) edge_flags: u32,
    // Total screen-space length of the segment, in pixels
    @location(4) @interpolate(flat) seg_len: f32
};

// Edge flags
const FLAG_SELECTED: u32 = (u32(1) << 0);
const FLAG_ACTIVE: u32 = (u32(1) << 1);
const FLAG_V0_SELECTED: u32 = (u32(1) << 2);
const FLAG_V1_SELECTED: u32 = (u32(1) << 3);
const FLAG_CUT: u32 = (u32(1) << 4);
const FLAG_BOUNDARY: u32 = (u32(1) << 5);
const FLAG_MOUNTAIN: u32 = (u32(1) << 6);
const FLAG_VALLEY: u32 = (u32(1) << 7);
const FLAG_HAS_FLAP: u32 = (u32(1) << 8);

// Calculates the colors of edges as would be seen on-screen.
fn _vs_color(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // Color the line (each vertex) based on its select status
    if (bool(in.flags & FLAG_ACTIVE)) { 
      out.color = theme.colors.element_active; 
    } else if (bool(in.flags & FLAG_SELECTED)) { 
      out.color = theme.colors.element_selected; 
    } else if ((in.offset.x == 0 && bool(in.flags & FLAG_V0_SELECTED)) || 
       (in.offset.x == 1 && bool(in.flags & FLAG_V1_SELECTED))) {
      out.color = theme.colors.element_selected; 
    }

    // Add the edge index for the selection engine
    out.select_idx = in.select_idx;
    return out;
}

// Calculates the colors of edge annotations (e.g. cut status) as would be seen on-screen.
fn _vs_color_thick(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    // Color the line based on input flags
    if (bool(in.flags & FLAG_BOUNDARY)) {
      out.color = theme.colors.edge_boundary;
    } else if (bool(in.flags & FLAG_CUT)) { 
      out.color = theme.colors.edge_cut;
    }

    return out;
}

// Calculates the clip position of edge vertices based on the width of the line
fn _vs_clip_pos(in: VertexInput, _out: VertexOutput, size: f32) -> VertexOutput {
    var out = _out;

    // Find screen-space positions of each vertex
    var clip_v0 = camera.view_proj * piece.affine * vec4<f32>(in.v0_pos, 1.0);
    var clip_v1 = camera.view_proj * piece.affine * vec4<f32>(in.v1_pos, 1.0);
    var screen_v0 = viewport.dimensions * (0.5 * clip_v0.xy / clip_v0.w + 0.5);
    var screen_v1 = viewport.dimensions * (0.5 * clip_v1.xy / clip_v1.w + 0.5);

    // Expand into line segment
    var basis_x = screen_v1 - screen_v0;
    var basis_y = normalize(vec2<f32>(-basis_x.y, basis_x.x));
    var pt = screen_v0 + in.offset.x * basis_x + (0.5 - in.offset.y) * basis_y * size;
    var clip = mix(clip_v0, clip_v1, in.offset.x);
    out.clip_position = _apply_depth_offset(
        vec4<f32>(clip.w * (2.0 * pt / viewport.dimensions - 1.0), clip.z, clip.w));

    // Carry the screen-space run of the line so `fs_fold` can stipple it. Inert
    // for every other fragment stage.
    out.seg_len = length(basis_x);
    out.dash_t = in.offset.x * out.seg_len;
    out.edge_flags = in.flags;

    // Move thick lines offscreen if not cut or boundary
    if (size == theme.sizes.line_width_thick && !bool(in.flags & (FLAG_CUT | FLAG_BOUNDARY))) { 
      out.clip_position.z = -100.0;
    }

    return out;
}

// [VS.1] Full mesh edges
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out = _vs_color(in, out);
    out = _vs_clip_pos(in, out, theme.sizes.line_width);
    return out;
}

// [VS.2] Thicker line edges (e.g. cut status)
@vertex
fn vs_cut(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out = _vs_color_thick(in, out);
    out = _vs_clip_pos(in, out, theme.sizes.line_width_thick);
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
    return in.color * vec4<f32>(1.0, 1.0, 1.0, 0.3);
}

// [FS.3] Select index rendering
@fragment
fn fs_select(in: VertexOutput) -> @location(0) vec4<u32> {
    return in.select_idx;
}

// Dash geometry, in screen-space pixels. A mountain is a short even dash and a
// valley a long dash-dot, so the two stay tellable apart even on a piece that
// only has one kind of fold on it. Keep the dash lengths — and the valley's
// dash / dot ratio — far apart if you retune these.
const MOUNTAIN_DASH_LEN: f32 = 7.0;
const VALLEY_DASH_LEN: f32 = 16.0;
const DASH_GAP: f32 = 7.0;
const DOT_LEN: f32 = 3.0;

// Scales a dash pattern so a whole number of periods fits the segment exactly,
// keeping dashes flush with the edge's vertices instead of clipped mid-stroke.
fn _fitted_scale(seg_len: f32, period: f32) -> f32 {
    let n = max(1.0, round(seg_len / period));
    return seg_len / (n * period);
}

// Tells whether a fragment falls on an inked part of the edge's fold pattern.
fn _fold_visible(flags: u32, t: f32, seg_len: f32) -> bool {
    let lineless = theme.sizes.fold_lines < 0.5;

    // Selection highlights stay legible as continuous lines, in both modes
    if (bool(flags & (FLAG_SELECTED | FLAG_ACTIVE))) { return true; }

    // Piece silhouette: the mesh boundary, or a cut edge with no flap on this
    // side. A cut that does carry a flap is the line the flap folds along, so
    // it's a fold and not part of the silhouette.
    if (bool(flags & FLAG_BOUNDARY) ||
        (bool(flags & FLAG_CUT) && !bool(flags & FLAG_HAS_FLAP))) { return true; }

    // Lineless mode draws nothing but the silhouette
    if (lineless) { return false; }

    // Mountain folds are a short even dash
    if (bool(flags & FLAG_MOUNTAIN)) {
        let base = MOUNTAIN_DASH_LEN + DASH_GAP;
        let s = _fitted_scale(seg_len, base);
        return (t % (base * s)) < MOUNTAIN_DASH_LEN * s;
    }

    // Valley folds are a longer dash followed by a dot
    if (bool(flags & FLAG_VALLEY)) {
        let base = VALLEY_DASH_LEN + DASH_GAP + DOT_LEN + DASH_GAP;
        let s = _fitted_scale(seg_len, base);
        let p = t % (base * s);
        return p < VALLEY_DASH_LEN * s
            || (p >= (VALLEY_DASH_LEN + DASH_GAP) * s
                && p < (VALLEY_DASH_LEN + DASH_GAP + DOT_LEN) * s);
    }

    // Flat enough to not be a fold at all, so no line
    return false;
}

// [FS.4] Fold-annotated rendering, for the piece / cutting view
@fragment
fn fs_fold(in: VertexOutput) -> @location(0) vec4<f32> {
    if (!_fold_visible(in.edge_flags, in.dash_t, in.seg_len)) { discard; }
    return in.color;
}
