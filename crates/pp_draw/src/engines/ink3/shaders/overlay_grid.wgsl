// Vertex shader
struct Camera { view_proj: mat4x4<f32>, view_proj_inv: mat4x4<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexInput {
   @builtin(vertex_index) vertex_index: u32
};

struct VertexOutput {
    @location(0) near_point: vec3<f32>,
    @location(1) far_point: vec3<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

/// Unprojects a point from screen space into world space
fn unproject_point(pt: vec2<f32>, z: f32) -> vec3<f32> {
    var unproj_point = camera.view_proj_inv * vec4<f32>(pt, z, 1.0);
    return unproj_point.xyz / unproj_point.w;
}

@vertex
fn vs_main(
    vert: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // Hard-code positions for each of the corners of the rect, indexed by vertex_index
    var OFFSETS = array<vec2<f32>, 4>(
        vec2<f32>(-1, -1),
        vec2<f32>(1, -1),
        vec2<f32>(-1, 1),
        vec2<f32>(1, 1)
    );

    var p = OFFSETS[vert.vertex_index];
    out.near_point = unproject_point(p, 0.0);
    out.far_point = unproject_point(p, 1.0);
    out.clip_position = vec4<f32>(p, 0.0, 1.0);
    return out;
}

// Fragment shader
@group(1) @binding(0) var depth_texture: texture_depth_multisampled_2d;

fn grid(pos: vec3<f32>, scale: f32) -> vec4<f32> {
    let fade_radius = 5.0;
    var distance = length(pos.xy);
    var fade = smoothstep(fade_radius, 0.0, distance);
    if fade > 0 {
        var coord = pos.xy * scale;
        var derivative = fwidth(coord);
        var grid = abs(fract(coord - 0.5) - 0.5) / derivative;
        var line = min(grid.x, grid.y);
        var minZ = min(derivative.y, 1.0);
        var minX = min(derivative.x, 1.0);
        var color = vec3<f32>(0.1, 0.1, 0.1);
        if abs(coord.x) < 0.05 {
            color.x = 1.0;
        } else if abs(coord.y) < 0.05 {
            color.y = 1.0;
        }
        return vec4<f32>(color, fade - min(line, fade));
    } else {
        return vec4<f32>(0, 0, 0, 0);
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var t = -in.near_point.z / (in.far_point.z - in.near_point.z);
    var world_pos = in.near_point + t * (in.far_point - in.near_point);
    // Convert world position to normalized device coordinates
    var clip_pos = camera.view_proj * vec4<f32>(world_pos, 1.0);
    var ndc_pos = clip_pos.xyz / clip_pos.w; // NDC [-1, 1]
    var screen_pos = vec2<f32>(
        (ndc_pos.x * 0.5 + 0.5),
        1 - (ndc_pos.y * 0.5 + 0.5)
    ) * camera.dimensions; // Map to screen space [0, dimensions]
    // Sample depth from the multisampled depth texture
    let screen_depth = textureLoad(depth_texture, vec2<i32>(screen_pos), 0);

    // Only render grid pixels in front of the sampled depth
    // return vec4<f32>(1.0, 1.0, 1.0, 1.0) * screen_depth;
    if ndc_pos.z >= screen_depth {
        discard;
    }

    return grid(world_pos, 2.0) * f32(t > 0);
}
