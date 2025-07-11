struct Viewport { position: vec2<f32>, dimensions: vec2<f32> };
@group(0) @binding(0) var<uniform> viewport: Viewport;
struct Camera { view_proj: mat4x4<f32>, eye: vec4<f32> };
@group(1) @binding(0) var<uniform> camera: Camera;

struct VertexInput {
   @location(0) offset: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) world_position: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Constants defining dimensions of the rectangle (will change)
    let PAD = 2.0;
    let DIMS = vec2<f32>(8.0, 8.0);

    var p = ((in.offset * (DIMS + PAD * 2)) - PAD) * vec2<f32>(1.0, -1.0);
    out.world_position = vec3<f32>(p, 0.0);
    out.clip_position = camera.view_proj * vec4<f32>(out.world_position, 1.0);
    return out;
}

// Fragment shader
fn grid(pos: vec3<f32>, scale: f32) -> vec4<f32> {
    let width = 8.0;
    let height = 8.0;
    let fade_radius = 1.5;
    var distance = length(pos.xy - clamp(pos.xy, vec2<f32>(0, -1 * height), vec2<f32>(width, 0)));
    var fade = smoothstep(fade_radius, 0.0, distance);
    // Scale the world-space position for the grid
    let coord = pos.xy * scale;
    // Compute screen-space derivatives for consistent line thickness
    let derivative = fwidth(coord);

    // Create grid lines by isolating fractional parts
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;

    // Determine line intensity with antialiasing
    let line = min(grid.x, grid.y);

    // Base grid color
    var axis_color = vec3<f32>(0.1, 0.1, 0.1);

    // Highlight axes (red for Y=0 (X axis), green for X=0 (Y axis))
    if abs(coord.y) < 0.05 {
        axis_color = vec3<f32>(0.3, 0.3, 0.3);
        // axis_color = vec3<f32>(1.0, 0.0, 0.0);
    } else if abs(coord.x) < 0.05 {
        axis_color = vec3<f32>(0.3, 0.3, 0.3);
        // axis_color = vec3<f32>(0.0, 1.0, 0.0);
    // Highlight bounds (red for Y=0 (X axis), green for X=0 (Y axis))
    } else if abs(coord.y + height * scale) < 0.05 || abs(coord.x - width * scale) < 0.05 {
        axis_color = vec3<f32>(0.3, 0.3, 0.3);
    }
    return vec4<f32>(axis_color, fade - min(line, fade));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return grid(in.world_position, 2.0);
}
