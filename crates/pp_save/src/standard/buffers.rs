use gltf_json::validation::Checked;
use gltf_json::{self as json, Value};

/// Helper for building GLTF buffers, buffer views, and accessors
pub struct GltfBufferBuilder {
    buffer: Vec<u8>,
    buffer_views: Vec<json::buffer::View>,
    accessors: Vec<json::Accessor>,
}

// Options to pass ino the accessor generators.
#[derive(Debug, Clone)]
pub struct AccessorOptions {
    pub normalized: bool,
    pub component_type: Checked<json::accessor::GenericComponentType>,
    pub type_: Checked<json::accessor::Type>,
    pub target: Option<Checked<json::buffer::Target>>,
    pub min: Option<gltf_json::Value>,
    pub max: Option<gltf_json::Value>,
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
        options: AccessorOptions,
    ) -> json::Index<json::Accessor> {
        let bytes = bytemuck::cast_slice(data);
        let buffer_view = self.add_buffer_view(bytes, options.target, None);

        let accessor = json::Accessor {
            buffer_view: Some(buffer_view),
            byte_offset: Some(0_u64.into()),
            count: (data.len() as u64).into(),
            component_type: options.component_type,
            min: options.min,
            max: options.max,
            normalized: options.normalized,
            type_: options.type_,
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

// ========== LOAD FUNCTIONS ==========

use crate::load::LoadError;

/// Reads typed data from a GLTF accessor
pub fn read_accessor<T: bytemuck::Pod + Copy>(
    buffers: &[gltf::buffer::Data],
    accessor: &gltf::Accessor,
) -> Result<Vec<T>, LoadError> {
    let buffer_view = accessor.view().ok_or(LoadError::Unknown)?;
    let offset = buffer_view.offset() + accessor.offset();
    let count = accessor.count();
    let stride = buffer_view.stride().unwrap_or(std::mem::size_of::<T>());

    let buffer = buffers.get(buffer_view.buffer().index()).ok_or(LoadError::Unknown)?;
    let mut result = Vec::with_capacity(count);

    if stride == std::mem::size_of::<T>() {
        // Tightly packed, can read directly
        let end = offset + count * std::mem::size_of::<T>();
        let slice = &buffer[offset..end];
        result.extend_from_slice(bytemuck::cast_slice(slice));
    } else {
        // Strided data, need to read element by element
        for i in 0..count {
            let elem_offset = offset + i * stride;
            let elem_bytes = &buffer[elem_offset..elem_offset + std::mem::size_of::<T>()];
            result.push(*bytemuck::from_bytes(elem_bytes));
        }
    }

    Ok(result)
}
