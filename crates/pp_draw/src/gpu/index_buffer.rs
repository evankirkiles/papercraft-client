use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct IndexBuf {
    pub label: String,
    /// The number of elements in the buffer (before casting to bytes)
    pub len: u32,

    /// The underlying GPU Buffer handle, or None if not yet written to
    buf: Option<wgpu::Buffer>,
    /// The size of the data currently in the `buf`
    size: u64,
}

impl IndexBuf {
    /// Creates an uninitialized Index Buffer.
    pub fn new(label: String) -> Self {
        Self { label, buf: None, size: 0, len: 0 }
    }

    /// Uploads data from the CPU into the GPU for this buffer, resizing if necessary
    pub fn update(&mut self, ctx: &super::Context, contents: &[impl bytemuck::Pod]) {
        self.len = contents.len() as u32;
        let bytes = bytemuck::cast_slice(contents);
        self.size = bytes.len() as u64;
        if let Some(buf) = self.buf.as_ref() {
            if buf.size() >= self.size {
                ctx.queue.write_buffer(buf, 0, bytes);
                return;
            }
        }
        self.buf = Some(ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(self.label.as_str()),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            contents: bytes,
        }));
    }

    /// Returns a slice handle into the buffer
    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buf.as_ref().unwrap().slice(0..self.size)
    }
}
