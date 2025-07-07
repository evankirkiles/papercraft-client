struct Viewport { position: vec2<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> viewport: Viewport;
struct Camera { view_proj: mat4x4<f32>, eye: vec4<f32> };
@group(1) @binding(0) var<uniform> camera: Camera;
struct Piece { affine: mat4x4<f32> };
@group(2) @binding(0) var<uniform> piece: Piece;
// Material for surface coloring / texturing / paper simulating
struct Material { base_color_factor: vec4<f32> };
@group(3) @binding(0) var<uniform> material: Material;
@group(3) @binding(1) var t_diffuse: texture_2d<f32>;
@group(3) @binding(2) var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

// Calculates the colors of surfaces as would be seen on-screen.
fn _vs_color(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.tex_coords = in.tex_coords;
    return out;
}

// Calculates the colors of surfaces as would be seen on-screen.
fn _vs_clip_pos(in: VertexInput, _out: VertexOutput) -> VertexOutput {
    var out = _out;
    out.clip_position = camera.view_proj * piece.affine * vec4<f32>(in.position, 1.0);
    return out;
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out = _vs_color(in, out);
    out = _vs_clip_pos(in, out);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords); 
    // return vec4<f32>(in.color * 1.0, 1.0) ;
    // return vec4<f32>(0.5, 0.5, 0.5, 1.0);
}

// [FS.2] Select masking
@fragment
fn fs_select(in: VertexOutput) -> @location(0) vec4<u32> {
    return vec4<u32>(0, 0, 0, 0) ;
}
