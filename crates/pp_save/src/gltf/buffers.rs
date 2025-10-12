use gltf_json as json;
use gltf_json::validation::Checked;

/// Helper for building GLTF buffers, buffer views, and accessors
pub struct GltfBufferBuilder {
    buffer: Vec<u8>,
    buffer_views: Vec<json::buffer::View>,
    accessors: Vec<json::Accessor>,
}

impl GltfBufferBuilder {
    pub fn new() -> Self {
        Self { buffer: Vec::new(), buffer_views: Vec::new(), accessors: Vec::new() }
    }

    /// Aligns the buffer to the specified byte boundary
    fn align_to(&mut self, alignment: usize) {
        let len = self.buffer.len();
        let remainder = len % alignment;
        if remainder != 0 {
            let padding = alignment - remainder;
            self.buffer.resize(len + padding, 0);
        }
    }

    /// Adds raw bytes to the buffer and returns a buffer view index
    pub fn add_buffer_view(
        &mut self,
        data: &[u8],
        target: Option<Checked<json::buffer::Target>>,
        byte_stride: Option<json::buffer::Stride>,
    ) -> json::Index<json::buffer::View> {
        // Align to 4-byte boundary for safety
        self.align_to(4);

        let byte_offset = self.buffer.len();
        let byte_length = data.len();
        self.buffer.extend_from_slice(data);

        let view = json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: gltf_json::validation::USize64(byte_length as u64),
            byte_offset: Some(gltf_json::validation::USize64(byte_offset as u64)),
            byte_stride,
            target,
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        };

        let index = self.buffer_views.len() as u32;
        self.buffer_views.push(view);
        json::Index::new(index)
    }

    /// Adds typed data to the buffer and creates an accessor
    pub fn add_accessor<T: bytemuck::Pod>(
        &mut self,
        data: &[T],
        component_type: Checked<json::accessor::GenericComponentType>,
        type_: Checked<json::accessor::Type>,
        target: Option<Checked<json::buffer::Target>>,
        normalized: bool,
    ) -> json::Index<json::Accessor> {
        let bytes = bytemuck::cast_slice(data);
        let buffer_view = self.add_buffer_view(bytes, target, None);

        let accessor = json::Accessor {
            buffer_view: Some(buffer_view),
            byte_offset: Some((0 as u64).into()),
            count: (data.len() as u64).into(),
            component_type,
            type_,
            min: None,
            max: None,
            normalized,
            sparse: None,
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        };

        let index = self.accessors.len() as u32;
        self.accessors.push(accessor);
        json::Index::new(index)
    }

    /// Builds the final GLTF components
    pub fn build(self) -> (Vec<json::Buffer>, Vec<json::buffer::View>, Vec<json::Accessor>) {
        let buffer = json::Buffer {
            byte_length: (self.buffer.len() as u64).into(),
            name: None,
            uri: Some(format!(
                "data:application/octet-stream;base64,{}",
                base64_encode(&self.buffer)
            )),
            extensions: Default::default(),
            extras: Default::default(),
        };

        (vec![buffer], self.buffer_views, self.accessors)
    }
}

fn base64_encode(data: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
}
