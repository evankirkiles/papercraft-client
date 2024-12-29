use crate::{cache, gpu};

mod modules;

pub struct InkEngine {
    module_surface: modules::SurfaceModule,
    module_wire_edge: modules::WireEdgeModule,
}

impl InkEngine {
    pub fn new(ctx: &gpu::Context) -> Self {
        let module_surface = modules::SurfaceModule::new(ctx);
        let module_wire_edge = modules::WireEdgeModule::new(ctx);
        Self { module_surface, module_wire_edge }
    }

    pub fn draw_mesh(&self, render_pass: &mut wgpu::RenderPass, mesh: &cache::MeshGPU) {
        // For module in the engine's modules:
        //  |- For pipeline in the module's pipeline (usually, 1)
        //      |- 1. Bind a batch (material EBO + VBO) for the pipeline
        //         2. Run the draw calls using the VBO and EBO
        //         3. Repeat

        // 1. Surface module (textured things)
        self.module_surface.draw_mesh(render_pass, mesh);
        self.module_wire_edge.draw_mesh(render_pass, mesh);
    }
}
