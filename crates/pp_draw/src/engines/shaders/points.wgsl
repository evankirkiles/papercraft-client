// Vertex shader
struct Camera { view_proj: mat4x4<f32>, resolution: vec2<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;

// Instanced rendering, so position corresponds to the instance's position
// and the vertex_index comes from the triangle strip defining the point rect.
struct VertexInput { 
  @location(0) offset: vec2<f32>,
  @location(1) pos: vec3<f32>,
  @location(2) flags: u32,
  @location(3) select_idx: vec2<u32>
};

struct VertexOutput { 
  @builtin(position) clip_position: vec4<f32>,
  @location(0) flags: u32,
  @location(1) select_idx: vec2<u32>
};

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    // The size of all sides of the square vertex dot
    var SIZE = 20.0;

    // Find screen-space positions of the vertex, offset by the instance pos
    var clip_center = camera.view_proj * vec4<f32>(in.pos, 1.0);
    var ndc_offset = SIZE * (0.5 - in.offset) / camera.resolution;
    out.clip_position = clip_center + vec4<f32>(ndc_offset * clip_center.w, 0.0, 0.0);
    out.flags = in.flags;
    out.select_idx = in.select_idx;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if bool(in.flags & 1) {
        return vec4<f32>(1.0, 0.5, 0.0, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}

@fragment
fn fs_select(in: VertexOutput) -> @location(0) vec2<u32> {
    return in.select_idx + vec2<u32>(1, 0);
}
