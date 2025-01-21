// Vertex shader
struct Camera { view_proj: mat4x4<f32>, resolution: vec2<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexInput {
  @location(0) offset: vec2<f32>,
  @location(1) v0_pos: vec3<f32>,
  @location(2) v1_pos: vec3<f32>,
  @location(3) flags: u32,
//   @location(4) select_idx: vec2<u32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Width of the line in pixels
    let SIZE = 1.5;

    // Find screen-space positions of each vertex
    var clip_v0 = camera.view_proj * vec4<f32>(in.v0_pos, 1.0);
    var clip_v1 = camera.view_proj * vec4<f32>(in.v1_pos, 1.0);
    var screen_v0 = camera.resolution * (0.5 * clip_v0.xy / clip_v0.w + 0.5);
    var screen_v1 = camera.resolution * (0.5 * clip_v1.xy / clip_v1.w + 0.5);

    // Expand into line segment
    var basis_x = screen_v1 - screen_v0;
    var basis_y = normalize(vec2<f32>(-basis_x.y, basis_x.x));
    var pt = screen_v0 + in.offset.x * basis_x + (0.5 - in.offset.y) * basis_y * SIZE;
    var clip = mix(clip_v0, clip_v1, in.offset.x);
    out.clip_position = vec4<f32>(clip.w * (2.0 * pt / camera.resolution - 1.0), clip.z, clip.w);
    out.color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let V0_SELECTED: u32 = u32(1) << 2;
    let V1_SELECTED: u32 = u32(1) << 3;
    if (in.offset.x == 0 && bool(in.flags & V0_SELECTED)) || (in.offset.x == 1 && bool(in.flags & V1_SELECTED)) {
        out.color = vec4<f32>(1.0, 0.5, 0.0, 1.0);
    }
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

@fragment
fn fs_select(in: VertexOutput) -> @location(0) vec4<u32> {
    return vec4<u32>(0, 0, 0, 0);
}
