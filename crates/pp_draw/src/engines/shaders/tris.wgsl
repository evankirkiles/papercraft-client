// Shared Camera uniform (2D or 3D)
struct Camera { view_proj: mat4x4<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;

// Per-piece uniform with piece-specific transforms
struct Piece { affine: mat4x4<f32> };
@group(1) @binding(0) var<uniform> piece: Piece;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) flags: u32,
    @location(3) select_idx: vec4<u32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) @interpolate(flat) select_idx: vec4<u32>
};

// Face flags (these are supplied per-vertex)
const FLAG_SELECTED: u32 = (u32(1) << 2);
const FLAG_ACTIVE: u32 = (u32(1) << 3);

// Calculates the colors of tris as would be seen on-screen.
fn _vs_color(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    // Color the line (each vertex) based on its select status
    if (bool(in.flags & FLAG_ACTIVE)) { 
      out.color = vec4<f32>(1.0, 1.0, 1.0, 0.2); 
    } else if (bool(in.flags & FLAG_SELECTED)) { 
      out.color = vec4<f32>(1.0, 0.5, 0.0, 0.2); 
    }

    // Add the edge index for the selection engine
    out.select_idx = in.select_idx;
    return out;
}

// Calculates the clip position of tri vertices, optionally with an affine
// transformation (e.g. to use for piece offsets)
fn _vs_clip_pos(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.clip_position = camera.view_proj * piece.affine * vec4<f32>(in.position, 1.0);
    return out;
}


// [VS.1] Full mesh
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
