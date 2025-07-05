struct Viewport { position: vec2<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> viewport: Viewport;
struct Camera { view_proj: mat4x4<f32>, eye: vec4<f32> };
@group(1) @binding(0) var<uniform> camera: Camera;
struct ToolTranslate { center_pos: vec2<f32> };
@group(2) @binding(0) var<uniform> tool: ToolTranslate;

struct VertexInput {
    @location(0) offset: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) line_uv: f32,          // 0 to 1 along the line
};

// How thick the line actually is
const LINE_WIDTH: f32 = 4.0;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pos = tool.center_pos + ((in.offset - 0.5) * LINE_WIDTH);

    // Convert to NDC
    let rel = (pos - viewport.position) / viewport.dimensions;
    let ndc = rel * 2.0 - 1.0;
    out.clip_position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0); // white dot
}
