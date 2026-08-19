struct ThemeSizes {
  line_width: f32,
  line_width_thick: f32,
  point_size: f32,
  fold_lines: f32,
  // Scales lengths this shader hardcodes in pixels, so they keep a
  // constant physical size as the pixel density changes.
  stroke_scale: f32,
  // Whether selected / active elements are highlighted at all. Off for
  // print, which must not bake transient editor state into the page.
  selection: f32,
};
struct ThemeColors {
  background: vec4<f32>,
  grid: vec4<f32>,
  grid_axis_x: vec4<f32>,
  grid_axis_y: vec4<f32>,
  element_active: vec4<f32>,
  element_selected: vec4<f32>,
  edge_cut: vec4<f32>,
  edge_boundary: vec4<f32>,
  // The default stroke color for edges carrying no annotation of their own.
  ink: vec4<f32>,
  padding: vec4<f32>,
};
struct Theme { sizes: ThemeSizes, colors: ThemeColors };
@group(0) @binding(0) var<uniform> theme: Theme;
struct Viewport { position: vec2<f32>, dimensions: vec2<f32> };
struct Camera { view_proj: mat4x4<f32>, eye: vec4<f32> };
@group(1) @binding(0) var<uniform> viewport: Viewport;
@group(1) @binding(1) var<uniform> camera: Camera;
struct ToolSelectPaint { center: vec2<f32>, radius: f32, _pad: f32 };
@group(2) @binding(0) var<uniform> tool: ToolSelectPaint;

const BORDER_THICKNESS: f32 = 1.0;
// The arc length of one dash, in screen-space pixels
const DASH_LENGTH: f32 = 5.0;

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

    // Expand the unit quad over the circle's bounding box, padded by the
    // border so the outermost ring pixels aren't clipped away
    let extent = tool.radius + BORDER_THICKNESS + 1.0;
    let screen_pos = mix(tool.center - extent, tool.center + extent, in.offset);
    out.local_pos = screen_pos;

    // Convert to viewport-relative NDC
    let rel = (screen_pos - viewport.position) / viewport.dimensions;
    let ndc = rel * 2.0 - 1.0;
    out.clip_position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let delta = in.local_pos - tool.center;
    let dist = length(delta);

    // Only the ring itself is drawn - the interior stays untouched so the
    // brush never washes out the geometry underneath it
    if (abs(dist - tool.radius) > BORDER_THICKNESS) {
        discard;
    }

    // Dash along the arc rather than along an axis, so dashes stay the same
    // length no matter how large the brush gets
    let arc = atan2(delta.y, delta.x) * tool.radius;
    if (floor(arc / DASH_LENGTH) % 2.0 != 0.0) {
        discard;
    }

    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
