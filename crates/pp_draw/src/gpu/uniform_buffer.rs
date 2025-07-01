use wgpu::util::DeviceExt;

/// A buffer for storing Uniform data to pass into shaders.
///
/// Because the BindingResource needed to create a BindGroup requires the buffer
/// directly, and wgpu does not support resizing of buffers (meaning we need to
/// create a new buffer any time we want to increase the size of a buffer), we'd
/// need to add some complexity to preserve the validity of BindGroups through
/// resized buffers.
///
/// Instead of worrying about this, we will just avoid thinking about variable
/// size UniformBuffers for now, and allocate the buffer with a pre-defined size
/// upon creation of a `UniformBuf` (so we can immediately use it for Bind Groups).
#[derive(Debug, Clone)]
pub struct UniformBuf {
    pub label: String,

    /// The underlying GPU Buffer handle
    buf: wgpu::Buffer,
}

impl UniformBuf {
    /// Creates a Uniform Vertex Buffer.
    pub fn new(ctx: &super::Context, label: String, size: usize) -> Self {
        Self {
            buf: ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label.as_str()),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            label,
        }
    }

    /// Creates a UBO with data to pre-populate it
    pub fn init(ctx: &super::Context, label: String, contents: &[impl bytemuck::Pod]) -> Self {
        Self {
            buf: ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label.as_str()),
                usage: wgpu::BufferUsages::UNIFORM,
                contents: bytemuck::cast_slice(contents),
            }),
            label,
        }
    }

    /// Uploads data from the CPU into the GPU for this buffer, resizing if necessary
    pub fn update(&mut self, ctx: &super::Context, contents: &[impl bytemuck::Pod]) {
        let bytes = bytemuck::cast_slice(contents);
        if bytes.len() != self.buf.size() as usize {
            panic!("Attempted to update UniformBuffer with a different size it was allocated with");
        }
        ctx.queue.write_buffer(&self.buf, 0, bytes);
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.buf.as_entire_binding()
    }
}
