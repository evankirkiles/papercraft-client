use pp_core::mesh::Mesh;
use pp_draw::select::SelectionQuery;
use std::sync::Arc;
use viewport::ViewportInput;
use winit::dpi::PhysicalPosition;
use winit::event_loop::EventLoopProxy;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, window::Window,
};

mod input;
mod viewport;

#[cfg(target_arch = "wasm32")]
use futures::channel::oneshot::Receiver;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
const CANVAS_ID: &str = "paperarium-engine";

/// Helps identify the active viewport for sending events to
#[derive(Debug, Default)]
enum ViewportType {
    D2,
    #[default]
    D3,
}

pub struct App {
    window: Option<Arc<Window>>,
    /// The size of the window
    size: PhysicalSize<u32>,
    /// The core state of the application
    state: pp_core::state::State,
    /// User Input state (e.g. buttons pressed)
    input_state: input::InputState,
    /// A reference to the viewport to use for sending input
    active_viewport: ViewportType,
    /// Manages synchronizing screen pixels with the app state
    renderer: Option<pp_draw::Renderer<'static>>,
    /// A proxy to the main event loop which this app runs on
    event_loop_proxy: EventLoopProxy<AppEvent>,
    /// Receiver for asynchronous winit setup in WASM
    #[cfg(target_arch = "wasm32")]
    renderer_receiver: Option<Receiver<pp_draw::Renderer<'static>>>,
}

pub enum AppSelectionAction {
    Nearest,
    NearestToggle,
    All,
}

pub enum AppEvent {
    Select { action: AppSelectionAction, query: SelectionQuery },
}

impl App {
    pub fn new(event_loop_proxy: EventLoopProxy<AppEvent>) -> Self {
        let mut app = Self {
            window: Default::default(),
            size: PhysicalSize { width: 1, height: 1 },
            state: Default::default(),
            input_state: Default::default(),
            renderer: Default::default(),
            active_viewport: Default::default(),
            event_loop_proxy,
            #[cfg(target_arch = "wasm32")]
            renderer_receiver: Default::default(),
        };
        let cube = Mesh::new_cube(0);
        app.state.meshes.insert(cube.id, cube);
        app
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn begin() {
    let event_loop: winit::event_loop::EventLoop<AppEvent> =
        winit::event_loop::EventLoop::with_user_event().build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new(event_loop.create_proxy());
    event_loop.run_app(&mut app).unwrap();
}

impl ApplicationHandler<AppEvent> for App {
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
                    self.renderer_receiver = Some(receiver);
                    #[cfg(feature = "console_error_panic_hook")]
                    console_error_panic_hook::set_once();
                    console_log::init_with_level(log::Level::Info)
                        .expect("Failed to initialize logger");
                    log::info!("Canvas dimensions: {canvas_width} x {canvas_height}");
                    wasm_bindgen_futures::spawn_local(async move {
                        let renderer = pp_draw::Renderer::new(
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
                    self.renderer = Some(pollster::block_on(async move {
                        pp_draw::Renderer::new(
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

    fn user_event(&mut self, _: &winit::event_loop::ActiveEventLoop, event: AppEvent) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        match event {
            AppEvent::Select { action, query } => {
                // Tell the select engine we received the query!
                renderer.recv_select_query(query);
                log::info!("received query: {:?}", query);
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
                    self.renderer = Some(renderer);
                    renderer_received = true;
                }
            }
            if renderer_received {
                self.renderer_receiver = None;
            }
        }

        let (Some(renderer), Some(window)) = (self.renderer.as_mut(), self.window.as_mut()) else {
            return;
        };

        if (match self.active_viewport {
            ViewportType::D2 => self.state.viewport_2d.handle_event(&event, &self.input_state),
            ViewportType::D3 => self.state.viewport_3d.handle_event(&event, &self.input_state),
        })
        .is_err()
        {
            return;
        }

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
                    self.input_state.mb1_pressed = state.is_pressed();
                    const SELECT_RADIUS: f64 = 30.0;
                    let cursor_pos = self.input_state.cursor_pos;
                    if !state.is_pressed() {
                        let mask = pp_draw::select::SelectionMask::POINTS;
                        let event_loop_proxy = self.event_loop_proxy.clone();
                        renderer
                            .send_select_query(
                                pp_draw::select::SelectionQuery {
                                    mask,
                                    rect: pp_draw::select::SelectionRect {
                                        x: (cursor_pos.x - SELECT_RADIUS).max(0.0) as u32,
                                        y: (cursor_pos.y - SELECT_RADIUS).max(0.0) as u32,
                                        width: SELECT_RADIUS as u32,
                                        height: SELECT_RADIUS as u32,
                                    },
                                },
                                move |query_state| {
                                    if let pp_draw::select::SelectManagerQueryState::Ready(query) =
                                        query_state
                                    {
                                        event_loop_proxy.send_event(AppEvent::Select {
                                            action: AppSelectionAction::Nearest,
                                            query,
                                        });
                                    }
                                },
                            )
                            .expect("select query failed!");
                    }
                }
                _ => {}
            },
            WindowEvent::CursorMoved { device_id: _, position } => {
                self.input_state.cursor_pos = position;
                self.active_viewport =
                    if (position.x as f32 / self.size.width as f32) < self.state.viewport_split {
                        ViewportType::D3
                    } else {
                        ViewportType::D2
                    };
            }
            WindowEvent::MouseWheel { phase, .. } => match phase {
                winit::event::TouchPhase::Started => self.input_state.is_touch = true,
                winit::event::TouchPhase::Ended => self.input_state.is_touch = false,
                _ => {}
            },
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                let (width, height) = ((width).max(1), (height).max(1));
                self.size.width = width;
                self.size.height = height;
                log::info!("Resizing renderer surface to: {width} x {height}");
                renderer.resize(width, height);
            }
            WindowEvent::CloseRequested => {
                log::info!("Close requested. Exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                renderer.sync(&mut self.state);
                renderer.draw();
            }
            _ => (),
        }

        window.request_redraw();
    }
}
