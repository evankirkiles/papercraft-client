// Vertex shader
struct Camera { view_proj: mat4x4<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;

// Instanced rendering, so position corresponds to the instance's position
// and the vertex_index comes from the triangle strip defining the point rect.
struct VertexInput { 
  @builtin(vertex_index) vertex_index: u32,
  @location(0) position: vec3<f32>,
  @location(1) flags: u32,
  @location(2) select_idx: vec2<u32>
};

struct VertexOutput { 
  @builtin(position) clip_position: vec4<f32>,
  @location(0) flags: u32,
  @location(1) select_idx: vec2<u32>
};


@vertex
fn vs_main(
    vert: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Hard-code positions for each of the corners of the rect, indexed by vertex_index
    var OFFSETS = array<vec2<f32>, 4>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5),
        vec2<f32>(-0.5, 0.5),
        vec2<f32>(0.5, 0.5)
    );

    // Width of the point in pixels
    var point_size = 20.0;
    // var offset = vert.offset * point_size;
    var offset = OFFSETS[vert.vertex_index] * point_size;
    var offset_mat = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(offset.x / camera.dimensions.x, offset.y / camera.dimensions.y, 0.0, 1.0)
    );

    out.clip_position = offset_mat * camera.view_proj * vec4<f32>(vert.position, 1.0);
    out.flags = vert.flags;
    out.select_idx = vert.select_idx;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if bool(in.flags & 1) {
        return vec4<f32>(1.0, 0.8, 0.0, 1.0);
    } else {
        return vec4<f32>(0.7, 0.7, 0.7, 1.0);
    }
}

@fragment
fn fs_select(in: VertexOutput) -> @location(0) vec2<u32> {
    return in.select_idx;
}
