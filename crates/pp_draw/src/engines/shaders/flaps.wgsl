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

// Rendering constants (to move to uniform)
const PIECE_DEPTH: f32 = 0.2;

// Edge flags
const E_FLAG_SELECTED: u32 = (u32(1) << 0);
const E_FLAG_ACTIVE: u32 = (u32(1) << 1);
const E_FLAG_V0_SELECTED: u32 = (u32(1) << 2);
const E_FLAG_V1_SELECTED: u32 = (u32(1) << 3);
const E_FLAG_CUT: u32 = (u32(1) << 4);

// Flap flags
const F_FLAG_EXISTS: u32 = (u32(1) << 0);

// Returns the 4 corners of the trapezoid flap
fn _compute_flap_corners(in: VertexInput) -> array<vec3<f32>, 4> {
    let v0 = in.v0_pos;
    let v1 = in.v1_pos;
    let v2 = in.v2_pos;

    // Get the direction in which the isosceles triangle extends
    let base_vec = v1 - v0;
    let base_len = length(base_vec);
    let base_dir = normalize(base_vec);
    let base_mid = 0.5 * (v0 + v1);
    let tri_normal = normalize(cross(base_vec, v2 - v0));
    let perp_dir = normalize(cross(tri_normal, base_dir));

    // Find the apex of the isosceles triangle in which we inscribe the flap 
    let angle0 = acos(clamp(dot(normalize(v1 - v0), normalize(v2 - v0)), -1.0, 1.0));
    let angle1 = acos(clamp(dot(normalize(v0 - v1), normalize(v2 - v1)), -1.0, 1.0));
    let min_angle = min(angle0, angle1);
    let height = 0.5 * base_len / tan(min_angle);
    let apex = base_mid + perp_dir * height;

    // Compute the short-edge vertices of the flap
    let top0 = v0 + (apex - v0) * PIECE_DEPTH;
    let top1 = v1 + (apex - v1) * PIECE_DEPTH;
    return array<vec3<f32>, 4>(v0, v1, top0, top1);  // bottom-left, bottom-right, top-right, top-left
}

// Calculates the colors of edges as would be seen on-screen.
fn _vs_color(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    // Color the flap (each vertex) based on its select status. Nonexistent
    // flaps should be clipped out already, but just in case...
    if (bool(in.flags & E_FLAG_SELECTED)) { 
      out.color = mix(out.color, vec4<f32>(1.0, 0.5, 0.0, 1.0), 0.5); 
    }

    // Add the edge index for the selection engine
    out.select_idx = in.select_idx;
    return out;
}

// Calculates the clip position of edge vertices based on the width of the line
fn _vs_clip_pos(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    
    // Compute corners of the flap trapezoid
    let corners = _compute_flap_corners(in);
  
    // Interpolate between corners of the flap based on input verts
    let base_pos = mix(corners[0], corners[1], in.offset.x);
    let top_pos = mix(corners[2], corners[3], in.offset.x);
    let pos = mix(base_pos, top_pos, in.offset.y);
    out.clip_position = camera.view_proj * piece.affine * vec4<f32>(pos, 1.0);

    // If flap doesn't exist, push it offscreen to avoid rasterization
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
