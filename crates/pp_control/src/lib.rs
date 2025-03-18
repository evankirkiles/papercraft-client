use ascii::ToAsciiChar;
use d2::Controller2D;
use d3::Controller3D;
use event::{EventContext, EventHandler, PressedState, UserEvent};
use keyboard::ModifierKeys;
use wasm_bindgen::prelude::*;

mod d2;
mod d3;
mod event;
mod keyboard;

use std::{cell::RefCell, ops::DerefMut, rc::Rc};

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct App {
    /// The core model of the App.
    state: Rc<RefCell<pp_core::State>>,
    /// The GPU resources of the App. Only created once a canvas is `attach`ed.
    renderer: Rc<RefCell<Option<pp_draw::Renderer<'static>>>>,
    /// Which viewport has "focus" and should take events
    active_viewport: Option<AppViewportType>,
    /// A shareable event context used across all event handlers
    event_context: EventContext,
    // Controllers for each viewport
    controller_3d: Controller3D,
    controller_2d: Controller2D,
}

/// "App" holds the entirey of the Rust application state. You can think of it
/// as the controller owning the Model (`pp_core`) and the View (`pp_draw`).
#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let state = Rc::new(RefCell::new(pp_core::State::default()));
        let renderer = Rc::new(RefCell::<Option<pp_draw::Renderer<'static>>>::new(None));
        Self {
            event_context: EventContext {
                state: state.clone(),
                renderer: renderer.clone(),
                ..Default::default()
            },
            state,
            renderer,
            ..Default::default()
        }
    }

    /// Attaches the Rust app to a canvas in the DOM. This allocates all the
    /// GPU resources the app might need. Actually drawing frames in a loop
    /// can then be done with `requestAnimationFrame` and the `draw` method.
    pub async fn attach(&mut self, canvas: JsValue) {
        if self.renderer.borrow().is_some() {
            return;
        };

        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into().expect("Failed to attach to canvas");
        let (width, height) = (canvas.width(), canvas.height());
        let target = wgpu::SurfaceTarget::Canvas(canvas);
        let renderer = pp_draw::Renderer::new(target, width, height).await;
        self.renderer.replace(Some(renderer));
    }

    /// De-allocates all the GPU resources for the app by "dropping" any renderer.
    pub fn unattach(&mut self) {
        self.renderer.replace(None);
    }

    // ---- RENDER CYCLE -----
    // Functions called in a loop or in a global listener, relevant to the renderer.

    /// Draws a single frame of the app to the canvas.
    pub fn draw(&mut self, _timestamp: u32) -> Result<(), JsError> {
        let mut state = self.state.borrow_mut();
        let mut renderer = self.renderer.borrow_mut();
        let renderer = renderer.as_mut().ok_or(AppError::NoCanvasAttached)?;
        renderer.select_poll(state.deref_mut());
        renderer.sync(state.deref_mut());
        renderer.draw();
        Ok(())
    }

    /// Resizes the virtual dimensions of the canvas.
    pub fn resize(&mut self, width: f64, height: f64, dpi: f64) -> Result<(), JsError> {
        let mut renderer = self.renderer.borrow_mut();
        let renderer = renderer.as_mut().ok_or(AppError::NoCanvasAttached)?;
        renderer.resize((width * dpi) as u32, (height * dpi) as u32);
        self.event_context.surface_size = event::PhysicalSize { width, height };
        self.event_context.surface_dpi = dpi;
        Ok(())
    }

    // ---- HOOKS ----
    // Functions that can be invoked by JavaScript on user interaction with HTML.

    pub fn update_horizontal_split(&mut self, frac: f64) {
        self.state.borrow_mut().viewport_split_x = frac;
    }

    pub fn update_vertical_split(&mut self, frac: f64) {
        self.state.borrow_mut().viewport_split_y = frac;
    }

    /// Returns the type viewport at the specified coordinates.
    fn get_viewport_at(&self, x: f64, y: f64) -> Option<AppViewportType> {
        let state = self.state.borrow();
        let (split_x, split_y) = (state.viewport_split_x, state.viewport_split_y);
        let frac_x = x / self.event_context.surface_size.width;
        let frac_y = y / self.event_context.surface_size.height;
        if !(0.0..=1.0).contains(&frac_x) || !(0.0..=1.0).contains(&frac_y) {
            None
        } else if (split_y..=1.0).contains(&frac_y) {
            Some(AppViewportType::D1)
        } else if (split_x..=1.0).contains(&frac_x) {
            Some(AppViewportType::D2)
        } else {
            Some(AppViewportType::D3)
        }
    }

    /// Internal function used to route an event to the viewport a user is currently
    /// interacting with, e.g. where their mouse is hovered. If the event still
    /// propagated, then the controller can maybe do some last-minute processing.
    fn handle_event(
        &mut self,
        ev: &UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        let Some(viewport) = self.active_viewport else { return Ok(Default::default()) };
        match viewport {
            AppViewportType::D2 => self.controller_2d.handle_event(&self.event_context, ev),
            AppViewportType::D3 => self.controller_3d.handle_event(&self.event_context, ev),
            _ => Ok(Default::default()),
        }
    }

    // ---- HANDLERS -----
    // Functions that are invoked directly by a JavaScript listener.

    pub fn handle_mouse_enter(
        &mut self,
        x: f64,
        y: f64,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        let curr_viewport = self.get_viewport_at(x, y);
        self.active_viewport = curr_viewport;
        self.event_context.last_mouse_pos = None;
        self.handle_event(&UserEvent::Pointer(event::PointerEvent::Enter))
    }

    pub fn handle_mouse_move(
        &mut self,
        x: f64,
        y: f64,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        let active_viewport = self.active_viewport;
        let curr_viewport = self.get_viewport_at(x, y);

        // If the user left an active viewport, notify the old viewport
        if let Some(active) = active_viewport {
            if curr_viewport.is_none_or(|curr| curr != active) {
                self.handle_event(&UserEvent::Pointer(event::PointerEvent::Exit))?;
            }
        }

        // If the user entered a new viewport, notify the new viewport
        if let Some(curr) = curr_viewport {
            if active_viewport.is_none_or(|active| curr != active) {
                self.active_viewport = Some(curr);
                self.event_context.last_mouse_pos = None;
                self.handle_event(&UserEvent::Pointer(event::PointerEvent::Enter))?;
            }
        }

        // Always emit the mouse move event to the most-recent viewport
        let pos = event::PhysicalPosition { x, y };
        self.event_context.last_mouse_pos = Some(pos);
        self.handle_event(&UserEvent::Pointer(event::PointerEvent::Move(pos)))
    }

    pub fn handle_mouse_exit(
        &mut self,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        let res = self.handle_event(&UserEvent::Pointer(event::PointerEvent::Exit));
        self.active_viewport = None;
        res
    }

    pub fn handle_wheel(
        &mut self,
        dx: f64,
        dy: f64,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        self.handle_event(&UserEvent::MouseWheel { dx: -dx, dy: -dy })
    }

    pub fn handle_modifiers_changed(&mut self, modifiers: u32) {
        self.event_context.modifiers = ModifierKeys::from_bits_truncate(modifiers);
    }

    /// Handles named key input.
    pub fn handle_named_key(
        &mut self,
        key: keyboard::NamedKey,
        pressed: PressedState,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        let key = keyboard::Key::Named(key);
        self.handle_event(&UserEvent::KeyboardInput(match pressed {
            PressedState::Pressed => event::KeyboardInputEvent::Down(key),
            PressedState::Unpressed => event::KeyboardInputEvent::Up(key),
        }))
    }

    /// Handles single-character keyboard input. This is so we can map to ASCII
    /// and not have to do string transformations across the WASM boundary.
    pub fn handle_key(
        &mut self,
        key: &str,
        pressed: PressedState,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        let key = keyboard::Key::from_key_code(key);
        self.handle_event(&UserEvent::KeyboardInput(match pressed {
            PressedState::Pressed => event::KeyboardInputEvent::Down(key),
            PressedState::Unpressed => event::KeyboardInputEvent::Up(key),
        }))
    }

    /// Handles clicks of all mouse buttons
    pub fn handle_mouse_button(
        &mut self,
        button: event::MouseButton,
        pressed: PressedState,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        self.handle_event(&UserEvent::MouseInput(match pressed {
            PressedState::Pressed => event::MouseInputEvent::Down(button),
            PressedState::Unpressed => event::MouseInputEvent::Up(button),
        }))
    }
}

/// An enum denoting which viewport is currently "active", e.g. being hovered
/// over, in the app.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
enum AppViewportType {
    D1,
    D2,
    D3,
}

#[derive(Debug, Clone)]
enum AppError {
    NoCanvasAttached,
}

impl std::error::Error for AppError {}
impl core::fmt::Display for AppError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Instruments Rust's logger with `console.log` capabilities
#[wasm_bindgen]
pub fn install_logging() {
    // Set up console logging / console error
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
}
