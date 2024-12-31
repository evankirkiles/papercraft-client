use pp_core::id::{self, Id};
use pp_core::mesh::Mesh;
use std::sync::Arc;
use winit::dpi::PhysicalPosition;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, window::Window,
};

mod input;

#[cfg(target_arch = "wasm32")]
use futures::channel::oneshot::Receiver;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
const CANVAS_ID: &str = "paperarium-engine";

pub struct App {
    window: Option<Arc<Window>>,
    /// The core state of the application
    state: pp_core::state::State,
    /// User Input state (e.g. buttons pressed)
    input_state: input::InputState,
    /// Which viewport to send inputs to (identified by Mouse Position or other)
    active_viewport: pp_core::id::ViewportId,
    /// Manages synchronizing screen pixels with the app state
    draw_manager: Option<pp_draw::DrawManager<'static>>,
    /// Receiver for asynchronous winit setup in WASM
    #[cfg(target_arch = "wasm32")]
    draw_manager_receiver: Option<Receiver<pp_draw::DrawManager<'static>>>,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            window: Default::default(),
            state: Default::default(),
            input_state: Default::default(),
            active_viewport: id::ViewportId::new(0),
            draw_manager: Default::default(),
            #[cfg(target_arch = "wasm32")]
            draw_manager_receiver: Default::default(),
        };
        let cube = Mesh::new_cube(0);
        app.state.meshes.insert(cube.id, cube);
        app
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn begin() {
    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        #[cfg_attr(not(target_arch = "wasm32"), allow(unused_mut))]
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
                    let (sender, receiver) = futures::channel::oneshot::channel();
                    self.draw_manager_receiver = Some(receiver);
                    #[cfg(feature = "console_error_panic_hook")]
                    console_error_panic_hook::set_once();
                    console_log::init_with_level(log::Level::Warn)
                        .expect("Failed to initialize logger");
                    log::info!("Canvas dimensions: {canvas_width} x {canvas_height}");
                    wasm_bindgen_futures::spawn_local(async move {
                        let renderer = pp_draw::DrawManager::new(
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
                    env_logger::builder()
                        .filter_level(log::LevelFilter::Debug)
                        .format_target(false)
                        .format_timestamp(None)
                        .init();
                    let inner_size = window_handle.inner_size();
                    self.draw_manager = Some(pollster::block_on(async move {
                        pp_draw::DrawManager::new(
                            window_handle.clone(),
                            inner_size.width,
                            inner_size.height,
                        )
                        .await
                    }));
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
            let mut draw_manager_received = false;
            if let Some(receiver) = self.draw_manager_receiver.as_mut() {
                if let Ok(Some(draw_manager)) = receiver.try_recv() {
                    self.draw_manager = Some(draw_manager);
                    draw_manager_received = true;
                }
            }
            if draw_manager_received {
                self.draw_manager_receiver = None;
            }
        }

        let (Some(draw_manager), Some(window)) = (self.draw_manager.as_mut(), self.window.as_mut())
        else {
            return;
        };

        // Handle event if GUI didn't do anything with it
        match event {
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                if key_code == winit::keyboard::KeyCode::Escape {
                    event_loop.exit()
                } else if key_code == winit::keyboard::KeyCode::ShiftLeft {
                    self.input_state.shift_pressed = state.is_pressed()
                }
            }
            WindowEvent::MouseInput { device_id: _, state, button } => match button {
                winit::event::MouseButton::Middle => {
                    self.input_state.mb3_pressed = state.is_pressed()
                }
                winit::event::MouseButton::Left => {
                    self.input_state.mb1_pressed = state.is_pressed()
                }
                _ => {}
            },
            WindowEvent::CursorMoved { device_id: _, position } => {
                if self.input_state.mb3_pressed {
                    let viewport = self.state.viewports.get_mut(&self.active_viewport).unwrap();
                    if self.input_state.shift_pressed {
                        viewport.camera.pan(
                            position.x - self.input_state.cursor_pos.x,
                            position.y - self.input_state.cursor_pos.y,
                        );
                    } else {
                        viewport.camera.orbit(
                            position.x - self.input_state.cursor_pos.x,
                            position.y - self.input_state.cursor_pos.y,
                        );
                    }
                }
                self.input_state.cursor_pos = position
            }
            WindowEvent::MouseWheel { device_id: _, delta, phase: _ } => {
                match delta {
                    // Standard scroll events should dolly in/out
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        let viewport = self.state.viewports.get_mut(&self.active_viewport).unwrap();
                        viewport.camera.dolly(y as f64);
                    }
                    // Touch "wheel" events should orbit
                    winit::event::MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                        let viewport = self.state.viewports.get_mut(&self.active_viewport).unwrap();
                        viewport.camera.orbit(x, y);
                    }
                }
            }
            WindowEvent::PinchGesture { device_id: _, delta, phase: _ } => {
                let viewport = self.state.viewports.get_mut(&self.active_viewport).unwrap();
                viewport.camera.dolly(delta);
            }
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                let (width, height) = ((width).max(1), (height).max(1));
                log::info!("Resizing renderer surface to: {width} x {height}");
                draw_manager.resize(width, height);
            }
            WindowEvent::CloseRequested => {
                log::info!("Close requested. Exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                draw_manager.sync(&mut self.state);
                draw_manager.draw().unwrap();
            }
            _ => (),
        }

        window.request_redraw();
    }
}
