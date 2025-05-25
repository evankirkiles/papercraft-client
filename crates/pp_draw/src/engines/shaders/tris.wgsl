// Vertex shader
struct Camera { view_proj: mat4x4<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) flags: u32,
    @location(3) select_idx: vec4<u32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) select_idx: vec4<u32>
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
 
    // Color vertices based on select status of entire face
    let FACE_SELECTED: u32 = u32(1) << 2;
    let FACE_ACTIVE: u32 = u32(1) << 3;
    out.color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    // Color tri based on select / active status
    if (bool(in.flags & FACE_SELECTED)) { out.color = vec4<f32>(1.0, 0.5, 0.0, 0.2); }
    if (bool(in.flags & FACE_ACTIVE)) { out.color = vec4<f32>(1.0, 1.0, 1.0, 0.2); }

    out.select_idx = in.select_idx;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

@fragment
fn fs_select(in: VertexOutput) -> @location(0) vec4<u32> {
    return in.select_idx + vec4<u32>(0, 0, 0, 1);
}
