use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use dom::DOMBindings;
use pp_core::mesh::Mesh;
use wasm_bindgen::prelude::*;
use web_sys::WheelEvent;

mod dom;

#[wasm_bindgen]
#[derive(Debug)]
pub struct App {
    /// The core model of the App.
    state: Rc<RefCell<pp_core::State>>,
    /// The GPU resources of the App. Only created once a canvas is `attach`ed.
    renderer: Option<Rc<RefCell<pp_draw::Renderer<'static>>>>,
    /// The surface onto which the App is rendered, and where to receive user input
    bindings: DOMBindings,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Self {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");

        let app = Self {
            state: Default::default(),
            bindings: DOMBindings::new(canvas_id),
            renderer: None,
        };

        // Set up basic initial scene
        let cube = Mesh::new_cube(0);
        app.state.borrow_mut().meshes.insert(cube.id, cube);
        app
    }

    pub async fn begin(&mut self) {
        if self.renderer.is_some() {
            return;
        };
        let window = self.bindings.window.handle.clone();
        let canvas = self.bindings.canvas.handle.clone();

        // Create the renderer (this is why we need async)
        let target = wgpu::SurfaceTarget::Canvas(canvas.deref().clone());
        let renderer = pp_draw::Renderer::new(target, canvas.width(), canvas.height()).await;
        let renderer = Rc::new(RefCell::new(renderer));

        // LISTENERS

        {
            let renderer = renderer.clone();
            let window = window.clone();
            let canvas = canvas.clone();
            let handler = move || {
                let dpi = window.device_pixel_ratio();
                let rect = canvas.deref().get_bounding_client_rect();
                let width = rect.width() * dpi;
                let height = rect.height() * dpi;
                renderer.borrow_mut().resize(width as u32, height as u32);
            };
            handler();
            self.bindings.window.on_resize(move |_: web_sys::Event| handler());
        }

        {
            let state = self.state.clone();
            self.bindings.canvas.on_wheel(move |e: WheelEvent| {
                let (dx, dy) = (-e.delta_x(), -e.delta_y());
                if e.alt_key() {
                    state.borrow_mut().viewport_3d.camera.dolly(dy * 0.5);
                } else if e.shift_key() {
                    state.borrow_mut().viewport_3d.camera.pan(dx, dy);
                } else {
                    state.borrow_mut().viewport_3d.camera.orbit(dx, dy);
                }
                e.dyn_into::<web_sys::Event>().unwrap().prevent_default();
            });
        }

        {
            // self.bindings.canvas.on_pointer_move(move |_: PointerEvent| log::info!("Cursor moved"));
        }

        self.renderer = Some(renderer)
    }

    pub fn draw(&mut self) {
        let Some(renderer_rc) = self.renderer.clone() else {
            return;
        };
        let mut renderer = renderer_rc.borrow_mut();
        renderer.sync(self.state.borrow_mut().deref_mut());
        renderer.draw();
    }

    pub fn resize_viewport(&mut self, frac: f32) {
        self.state.borrow_mut().viewport_split = frac;
    }
}
