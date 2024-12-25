use crate::gpu;

mod modules;

pub struct InkEngine {
    module_geometry: modules::GeometryModule,
}

impl InkEngine {
    pub fn new(ctx: &gpu::GPUContext) -> Self {
        let module_geometry = modules::GeometryModule::new(ctx);
        Self { module_geometry }
    }

    pub fn resize(&mut self) {}

    pub fn begin_sync(&self) {
        self.module_geometry.begin_sync();
    }

    pub fn draw_sync(
        &self,
        render_pass: &mut wgpu::RenderPass,
        // mesh: &resources::GPUModel,
    ) {
        // For module in the engine's modules:
        //  |- For pipeline in the module's pipeline (usually, 1)
        //      |- 1. Bind a batch (material EBO + VBO) for the pipeline
        //         2. Run the draw calls using the VBO and EBO
        //         3. Repeat

        // 1. Geometry module
        self.module_geometry.sync_model(render_pass);
    }

    pub fn end_sync(&self) {
        self.module_geometry.end_sync();
    }
}
