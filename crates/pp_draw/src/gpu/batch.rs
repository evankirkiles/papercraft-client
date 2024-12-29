// use wgpu::util::DeviceExt;

use std::{cell::RefCell, rc::Rc};

/// A "Batch" contains a group of buffers needed for a single draw call.
#[derive(Clone)]
pub struct Batch {
    /// All vertex buffers in the batch (pos, normals, etc.).
    /// Requires at least the pos buffer (verts[0])
    pub vbos: Vec<Rc<RefCell<super::VertBuf>>>,
    /// Elements used to index into the buffer
    pub ibo: Rc<RefCell<super::IndexBuf>>,
}

impl Batch {
    pub fn bind(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_index_buffer(self.ibo.borrow().slice(), wgpu::IndexFormat::Uint32);
        for (i, vbo) in self.vbos.iter().enumerate() {
            render_pass.set_vertex_buffer(i as u32, vbo.borrow().slice());
        }
    }

    pub fn draw_indexed(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.draw_indexed(0..self.ibo.borrow().len, 0, 0..1);
    }
}
