struct Viewport { position: vec2<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> viewport: Viewport;
struct Camera { view_proj: mat4x4<f32>, eye: vec4<f32> };
@group(1) @binding(0) var<uniform> camera: Camera;
struct ToolSelectBox { start_pos: vec2<f32>, end_pos: vec2<f32> };
@group(2) @binding(0) var<uniform> tool: ToolSelectBox;

struct VertexInput {
   @location(0) offset: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Normalize tool positions
    let min_pos = min(tool.start_pos, tool.end_pos);
    let max_pos = max(tool.start_pos, tool.end_pos);

    // Interpolate inside screen-space bounds
    let screen_pos = mix(min_pos, max_pos, in.offset);
    out.local_pos = screen_pos;

    // Convert to viewport-relative NDC
    let rel = (screen_pos - viewport.position) / viewport.dimensions;
    let ndc = rel * 2.0 - 1.0;
    out.clip_position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let min_pos = min(tool.start_pos, tool.end_pos);
    let max_pos = max(tool.start_pos, tool.end_pos);
    let border_thickness = 1.0; // in screen-space units
    let dash_size = 5.0;

    // Check if the fragment is near an edge (border)
    let inside = all(in.local_pos >= min_pos) && all(in.local_pos <= max_pos);
    let near_left   = abs(in.local_pos.x - min_pos.x) < border_thickness;
    let near_right  = abs(in.local_pos.x - max_pos.x) < border_thickness;
    let near_top    = abs(in.local_pos.y - min_pos.y) < border_thickness;
    let near_bottom = abs(in.local_pos.y - max_pos.y) < border_thickness;

    let is_border = (near_left || near_right || near_top || near_bottom);

    // Dotted/dashed effect using screen-space coordinate
    var dashed = true;
    if (near_top || near_bottom) {
        dashed = floor(in.local_pos.x / dash_size) % 2.0 == 0.0;
    } else if (near_left || near_right) {
        dashed = floor(in.local_pos.y / dash_size) % 2.0 == 0.0;
    }

    if (is_border && dashed) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0); // solid white
    } else {
        return vec4<f32>(1.0, 1.0, 1.0, 0.01); // translucent white fill
    }
}
