// Shared Camera uniform (2D or 3D)
struct Camera { view_proj: mat4x4<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;

// Per-piece uniform with piece-specific transforms
struct Piece { affine: mat4x4<f32> };
@group(1) @binding(0) var<uniform> piece: Piece;

struct VertexInput {
    @location(0) offset: vec2<f32>,
    @location(1) v0_pos: vec3<f32>,
    @location(2) v1_pos: vec3<f32>,
    @location(3) v2_pos: vec3<f32>,
    @location(4) flap_flags: u32,
    @location(5) flags: u32,
    @location(6) select_idx: vec2<u32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) select_idx: vec2<u32>
};

// Piece depth
const PIECE_DEPTH: f32 = 0.2;

// Edge flags
const E_FLAG_SELECTED: u32 = (u32(1) << 0);
const E_FLAG_ACTIVE: u32 = (u32(1) << 1);
const E_FLAG_V0_SELECTED: u32 = (u32(1) << 2);
const E_FLAG_V1_SELECTED: u32 = (u32(1) << 3);
const E_FLAG_CUT: u32 = (u32(1) << 4);

// Flap flags
const F_FLAG_EXISTS: u32 = (u32(1) << 0);

// Calculates the colors of edges as would be seen on-screen.
fn _vs_color(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    // Color the flap (each vertex) based on its select status. Nonexistent
    // flaps should be clipped out already, but just in case...
    if (bool(in.flap_flags ^ F_FLAG_EXISTS)) { 
      out.color = vec4<f32>(1.0, 1.0, 1.0, 0.0); 
    }

    // Add the edge index for the selection engine
    out.select_idx = in.select_idx;
    return out;
}

// Calculates the clip position of edge vertices based on the width of the line
fn _vs_clip_pos(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    
let u = in.offset.x;
    let v = in.offset.y;

    let v0_0 = in.v0_pos;
    let v0_1 = in.v1_pos;
    let v2 = in.v2_pos;

    let base_vec = v0_1 - v0_0;
    let base_len = length(base_vec);
    let base_dir = normalize(base_vec);
    let base_mid = 0.5 * (v0_0 + v0_1);

    // Triangle normal
    let tri_normal = normalize(cross(base_vec, v2 - v0_0));

    // Use the triangle normal and base direction to get a perpendicular vector in the triangle plane
    let perp_dir = normalize(cross(tri_normal, base_dir));

    // Compute angles at v0 and v1
    let angle0 = acos(clamp(dot(normalize(v0_1 - v0_0), normalize(v2 - v0_0)), -1.0, 1.0));
    let angle1 = acos(clamp(dot(normalize(v0_0 - v0_1), normalize(v2 - v0_1)), -1.0, 1.0));
    let min_angle = min(angle0, angle1);

    // Height of isosceles triangle with base length and base angle
    let height = 0.5 * base_len / tan(min_angle);
    let apex = base_mid + perp_dir * height;
    let v1_0 = v0_0 + (apex - v0_0) * PIECE_DEPTH;
    let v1_1 = v0_1 + (apex - v0_1) * PIECE_DEPTH;

    // Interpolate between base and apex to create trapezoid
    let base_pos = mix(v0_0, v0_1, in.offset.x);
    let top_pos = mix(v1_0, v1_1, in.offset.x);
    let pos = mix(base_pos, top_pos, in.offset.y);
    out.clip_position = camera.view_proj * piece.affine * vec4<f32>(pos, 1.0);

    if (bool(in.flap_flags ^ F_FLAG_EXISTS)) {
        out.clip_position.z = -100.0;
    }

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

// [FS.1] Normal rendering
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// [FS.2] Select index rendering
@fragment
fn fs_select(in: VertexOutput) -> @location(0) vec4<u32> {
    return vec4<u32>(0, 0, in.select_idx + vec2<u32>(0, 1));
}
