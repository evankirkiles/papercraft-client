use std::sync::Arc;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent,
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use futures::channel::oneshot::Receiver;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

const CANVAS_ID: &str = "paperarium-engine";

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<pp_renderer::Renderer<'static>>,
    viewports: Vec<pp_renderer::viewport_3d::Viewport3d>,
    #[cfg(target_arch = "wasm32")]
    renderer_receiver: Option<Receiver<pp_renderer::Renderer<'static>>>,
    last_size: (u32, u32),
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut attributes = Window::default_attributes();

        #[allow(unused_assignments)]
        #[cfg(target_arch = "wasm32")]
        let (mut canvas_width, mut canvas_height) = (0, 0);

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowAttributesExtWebSys;
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id(CANVAS_ID)
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            canvas_width = canvas.width();
            canvas_height = canvas.height();
            self.last_size = (canvas_width, canvas_height);
            attributes = attributes.with_canvas(Some(canvas));
        }

        if let Ok(window) = event_loop.create_window(attributes) {
            let first_window_handle = self.window.is_none();
            let window_handle = Arc::new(window);
            self.window = Some(window_handle.clone());

            // First-time window setup
            if first_window_handle {
                #[cfg(target_arch = "wasm32")]
                {
                    let (sender, receiver) =
                        futures::channel::oneshot::channel();
                    self.renderer_receiver = Some(receiver);
                    #[cfg(feature = "console_error_panic_hook")]
                    console_error_panic_hook::set_once();
                    console_log::init_with_level(log::Level::Warn)
                        .expect("Failed to initialize logger");
                    log::info!(
                        "Canvas dimensions: {canvas_width} x {canvas_height}"
                    );
                    wasm_bindgen_futures::spawn_local(async move {
                        let renderer = pp_renderer::Renderer::new(
                            window_handle.clone(),
                            canvas_width,
                            canvas_height,
                        )
                        .await;
                        if sender.send(renderer).is_err() {
                            log::error!("Failed to create and send renderer!");
                        }
                    });
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    env_logger::init();
                    let inner_size = window_handle.inner_size();
                    self.last_size = (inner_size.width, inner_size.height);
                    let renderer = pollster::block_on(async move {
                        pp_renderer::Renderer::new(
                            window_handle.clone(),
                            inner_size.width,
                            inner_size.height,
                        )
                        .await
                    });
                    let half_width = inner_size.width as f32 / 2.0;
                    let half_height = inner_size.height as f32 / 2.0;
                    for i in 0..2 {
                        self.viewports.push(renderer.add_viewport(
                            pp_renderer::viewport_3d::Rect {
                                x: i as f32 * half_width,
                                y: i as f32 * half_height,
                                w: half_width,
                                h: half_height,
                            },
                        ));
                    }
                    self.renderer = Some(renderer);
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // Catch the new window created asynchronously on web
        #[cfg(target_arch = "wasm32")]
        {
            let mut renderer_received = false;
            if let Some(receiver) = self.renderer_receiver.as_mut() {
                if let Ok(Some(renderer)) = receiver.try_recv() {
                    self.viewports.push(renderer.add_viewport(
                        pp_renderer::viewport_3d::Rect {
                            x: 0.0,
                            y: 0.0,
                            w: 100.0,
                            h: 100.0,
                        },
                    ));
                    self.renderer = Some(renderer);
                    renderer_received = true;
                }
            }
            if renderer_received {
                self.renderer_receiver = None;
            }
        }

        let (Some(renderer), Some(window)) =
            (self.renderer.as_mut(), self.window.as_mut())
        else {
            return;
        };

        // Handle event if GUI didn't do anything with it
        match event {
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key:
                            winit::keyboard::PhysicalKey::Code(key_code),
                        ..
                    },
                ..
            } => {
                if matches!(key_code, winit::keyboard::KeyCode::Escape) {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                let (width, height) = ((width).max(1), (height).max(1));
                log::info!("Resizing renderer surface to: {width} x {height}");
                renderer.resize(width, height);
                self.last_size = (width, height);
            }
            WindowEvent::CloseRequested => {
                log::info!("Close requested. Exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                renderer.render(self.viewports.as_slice()).unwrap();
            }
            _ => (),
        }

        window.request_redraw();
    }
}