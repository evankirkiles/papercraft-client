mod model;
mod renderer;
mod resources;
mod texture;

use renderer::Renderer;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

const CANVAS_ID: &str = "paperarium-engine";

// Include web-specific crates if necessary
cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
        use winit::platform::web::WindowBuilderExtWebSys;
    }
}

// The main entry point for the engine
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run() {
    // Set up initial logging
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            #[cfg(feature = "console_error_panic_hook")]
            console_error_panic_hook::set_once();
            console_log::init_with_level(log::Level::Warn).expect("Failed to initialize logger");
        } else {
            env_logger::init();
        }
    }

    // Configure the window builder per-platform
    let mut builder = WindowBuilder::new();
    #[cfg(target_arch = "wasm32")]
    {
        // Get the canvas to draw to
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(CANVAS_ID)
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        builder = builder.with_canvas(Some(canvas));
    }

    // Set up the window
    let event_loop = EventLoop::new().unwrap();
    let window = builder.build(&event_loop).unwrap();
    let mut renderer = Renderer::new(&window).await;
    let mut surface_configured = false;

    // Now run the event loop
    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == renderer.window().id() => {
                if !renderer.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            log::info!("physical_size: {physical_size:?}");
                            surface_configured = true;
                            renderer.resize(*physical_size)
                        }
                        WindowEvent::RedrawRequested => {
                            // This tells winit that we want another frame after this one
                            renderer.window().request_redraw();
                            if !surface_configured {
                                return;
                            }

                            renderer.update();
                            match renderer.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    renderer.resize(renderer.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    log::error!("OutOfMemory");
                                    control_flow.exit();
                                }
                                // This happens when the a frame takes too long to present
                                Err(wgpu::SurfaceError::Timeout) => {
                                    log::warn!("Surface timeout")
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        })
        .unwrap();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn begin() {
    wasm_bindgen_futures::spawn_local(run());
}
