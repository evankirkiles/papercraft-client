// Vertex shader
struct Camera { view_proj: mat4x4<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;

// Per-piece uniform with piece-specific transforms
struct Piece { affine: mat4x4<f32> };
@group(1) @binding(0) var<uniform> piece: Piece;

// Instanced rendering, so position corresponds to the instance's position
// and the vertex_index comes from the triangle strip defining the point rect.
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

// Point size (width in pixels of sides of vertex dot squares)
const POINT_SIZE: f32 = 16.0;

// Vertex flags
const FLAG_SELECTED: u32 = (u32(1) << 0);
const FLAG_ACTIVE: u32 = (u32(1) << 1);

// Calculates the colors of vertices as would be seen on-screen.
fn _vs_color(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    // Color each vertex based on its select status
    if (bool(in.flags & FLAG_ACTIVE)) {
      out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else if (bool(in.flags & FLAG_SELECTED)) {
      out.color = vec4<f32>(1.0, 0.5, 0.0, 1.0);
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
    var ndc_offset = POINT_SIZE * (0.5 - in.offset) / camera.dimensions;
    out.clip_position = (clip_center + vec4<f32>(ndc_offset * clip_center.w, 0.0, 0.0));
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

// [FS.2] Select index rendering
@fragment
fn fs_select(in: VertexOutput) -> @location(0) vec4<u32> {
    return in.select_idx + vec4<u32>(0, 0, 0, 1);
}
