struct Viewport { position: vec2<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> viewport: Viewport;
struct Camera { view_proj: mat4x4<f32>, eye: vec4<f32> };
@group(1) @binding(0) var<uniform> camera: Camera;
struct ToolTranslate { center_pos: vec2<f32>, flags: u32 };
@group(2) @binding(0) var<uniform> tool: ToolTranslate;

struct VertexInput {
    @location(0) offset: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) line_uv: f32,          // 0 to 1 along the line
};

// How thick the line actually is
const LINE_WIDTH: f32 = 1.5;

// Flags for the tool
const FLAG_X_LOCKED: u32 = (u32(1) << 0);
const FLAG_Y_LOCKED: u32 = (u32(1) << 1);

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pos = tool.center_pos + ((in.offset - 0.5) * LINE_WIDTH);
    let rel = (pos - viewport.position) / viewport.dimensions;
    var ndc = rel * 2.0 - 1.0;

    if (bool(tool.flags & FLAG_X_LOCKED)) { 
      ndc.x = (in.offset.x - 0.5) * 2.0;
    } else if (bool(tool.flags & FLAG_Y_LOCKED)) {
      ndc.y = (in.offset.y - 0.5) * 2.0;
    }

    out.clip_position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (bool(tool.flags & FLAG_X_LOCKED)) { 
      return vec4<f32>(1.0, 0.0, 0.0, 0.2);
    } else if (bool(tool.flags & FLAG_Y_LOCKED)) {
      return vec4<f32>(0.0, 1.0, 0.0, 0.2);
    } else {
      discard;
      return vec4<f32>(0.0, 1.0, 0.0, 0.0);;
    }
}
