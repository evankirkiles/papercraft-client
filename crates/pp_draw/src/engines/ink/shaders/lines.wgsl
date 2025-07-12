struct Viewport { position: vec2<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> viewport: Viewport;
struct Camera { view_proj: mat4x4<f32>, eye: vec4<f32> };
@group(1) @binding(0) var<uniform> camera: Camera;
struct Piece { affine: mat4x4<f32> };
@group(2) @binding(0) var<uniform> piece: Piece;

struct VertexInput {
  @location(0) offset: vec2<f32>,
  @location(1) v0_pos: vec3<f32>,
  @location(2) v1_pos: vec3<f32>,
  @location(3) flags: u32,
  @location(4) select_idx: vec4<u32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) select_idx: vec4<u32>
};

// Colors (opaque for overlays)
const COLOR_ACTIVE: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
const COLOR_SELECTED: vec3<f32> = vec3<f32>(1.0, 0.5, 0.0);
const COLOR_BOUNDARY: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);
const COLOR_CUT: vec3<f32> = vec3<f32>(1.0, 0.0, 0.0);

// Line width
const LINE_WIDTH_THIN: f32 = 1.5;
const LINE_WIDTH_THICK: f32 = 5.0;

// Edge flags
const FLAG_SELECTED: u32 = (u32(1) << 0);
const FLAG_ACTIVE: u32 = (u32(1) << 1);
const FLAG_V0_SELECTED: u32 = (u32(1) << 2);
const FLAG_V1_SELECTED: u32 = (u32(1) << 3);
const FLAG_CUT: u32 = (u32(1) << 4);
const FLAG_BOUNDARY: u32 = (u32(1) << 5);

// Calculates the colors of edges as would be seen on-screen.
fn _vs_color(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // Color the line (each vertex) based on its select status
    if (bool(in.flags & FLAG_ACTIVE)) { 
      out.color = vec4<f32>(COLOR_ACTIVE, 1.0); 
    } else if (bool(in.flags & FLAG_SELECTED)) { 
      out.color = vec4<f32>(COLOR_SELECTED, 1.0);
    } else if ((in.offset.x == 0 && bool(in.flags & FLAG_V0_SELECTED)) || 
       (in.offset.x == 1 && bool(in.flags & FLAG_V1_SELECTED))) {
      out.color = vec4<f32>(COLOR_SELECTED, 1.0);
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
      out.color = vec4<f32>(COLOR_BOUNDARY, 1.0);
    } else if (bool(in.flags & FLAG_CUT)) { 
      out.color = vec4<f32>(COLOR_CUT, 1.0);
    }

    return out;
}

// Calculates the clip position of edge vertices based on the width of the line
fn _vs_clip_pos(in: VertexInput, _out: VertexOutput, size: f32) -> VertexOutput {
    var out = _out;

    // Move lines slightly towards camera (by 0.01 in world space)
    var v0 = in.v0_pos + normalize(camera.eye.xyz - in.v0_pos) * camera.eye.w * 0.001;
    var v1 = in.v1_pos + normalize(camera.eye.xyz - in.v1_pos) * camera.eye.w * 0.001;
    
    // Find screen-space positions of each vertex
    var clip_v0 = camera.view_proj * piece.affine * vec4<f32>(v0, 1.0);
    var clip_v1 = camera.view_proj * piece.affine * vec4<f32>(v1, 1.0);
    var screen_v0 = viewport.dimensions * (0.5 * clip_v0.xy / clip_v0.w + 0.5);
    var screen_v1 = viewport.dimensions * (0.5 * clip_v1.xy / clip_v1.w + 0.5);

    // Expand into line segment
    var basis_x = screen_v1 - screen_v0;
    var basis_y = normalize(vec2<f32>(-basis_x.y, basis_x.x));
    var pt = screen_v0 + in.offset.x * basis_x + (0.5 - in.offset.y) * basis_y * size;
    var clip = mix(clip_v0, clip_v1, in.offset.x);
    out.clip_position = vec4<f32>(clip.w * (2.0 * pt / viewport.dimensions - 1.0), clip.z, clip.w);

    // Move thick lines offscreen if not cut or boundary
    if (size == LINE_WIDTH_THICK && !bool(in.flags & (FLAG_CUT | FLAG_BOUNDARY))) { 
      out.clip_position.z = -100.0;
    }

    return out;
}

// [VS.1] Full mesh edges
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out = _vs_color(in, out);
    out = _vs_clip_pos(in, out, LINE_WIDTH_THIN);
    return out;
}

// [VS.2] Thicker line edges (e.g. cut status)
@vertex
fn vs_cut(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out = _vs_color_thick(in, out);
    out = _vs_clip_pos(in, out, LINE_WIDTH_THICK);
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
