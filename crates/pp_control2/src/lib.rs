use d2::Controller2D;
use d3::Controller3D;
use event::{EventContext, EventHandler, EventModifierKeys, UserEvent};
use wasm_bindgen::prelude::*;

mod d2;
mod d3;
mod event;

use std::{cell::RefCell, ops::DerefMut, rc::Rc};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
enum AppViewport {
    D1,
    D2,
    D3,
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct App {
    /// The core model of the App.
    state: Rc<RefCell<pp_core::State>>,
    /// The GPU resources of the App. Only created once a canvas is `attach`ed.
    renderer: Rc<RefCell<Option<pp_draw::Renderer<'static>>>>,
    /// Which viewport has "focus" and should take events
    active_viewport: Option<AppViewport>,
    /// A shareable event context used across all event handlers
    event_context: EventContext,
    // Controllers for each viewport
    controller_3d: Controller3D,
    controller_2d: Controller2D,
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

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Set up console logging / console error
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
        let state = Rc::new(RefCell::new(pp_core::State::default()));
        let renderer = Rc::new(RefCell::<Option<pp_draw::Renderer<'static>>>::new(None));

        let event_context = EventContext::default();
        let controller_3d = Controller3D::new(state.clone(), renderer.clone());
        let controller_2d = Controller2D::new(state.clone(), renderer.clone());
        Self { state, renderer, event_context, controller_3d, controller_2d, active_viewport: None }
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

    /// Draws a single frame of the app to the canvas.
    pub fn draw(&mut self, _timestamp: u32) -> Result<(), JsError> {
        let mut renderer = self.renderer.borrow_mut();
        let renderer = renderer.as_mut().ok_or(AppError::NoCanvasAttached)?;
        renderer.sync(self.state.borrow_mut().deref_mut());
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

    pub fn update_horizontal_split(&mut self, frac: f64) {
        self.state.borrow_mut().viewport_split_x = frac;
    }

    pub fn update_vertical_split(&mut self, frac: f64) {
        self.state.borrow_mut().viewport_split_y = frac;
    }

    /// Returns the viewport at the specified coordinates.
    fn get_viewport_at(&self, x: f64, y: f64) -> Option<AppViewport> {
        let state = self.state.borrow();
        let (split_x, split_y) = (state.viewport_split_x, state.viewport_split_y);
        let frac_x = x / self.event_context.surface_size.width;
        let frac_y = y / self.event_context.surface_size.height;
        if !(0.0..=1.0).contains(&frac_x) || !(0.0..=1.0).contains(&frac_y) {
            None
        } else if (split_y..=1.0).contains(&frac_y) {
            Some(AppViewport::D1)
        } else if (split_x..=1.0).contains(&frac_x) {
            Some(AppViewport::D2)
        } else {
            Some(AppViewport::D3)
        }
    }

    /// Internal function used to route an event to the viewport a user is currently
    /// interacting with, e.g. where their mouse is hovered.
    fn send_event_to_active_viewport(
        &mut self,
        ev: &UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        let Some(viewport) = self.active_viewport else { return Ok(Default::default()) };
        match viewport {
            AppViewport::D2 => self.controller_2d.handle_event(&self.event_context, ev),
            AppViewport::D3 => self.controller_3d.handle_event(&self.event_context, ev),
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
        self.send_event_to_active_viewport(&UserEvent::Mouse(event::MouseEvent::Enter))?;
        Ok(Default::default())
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
                self.send_event_to_active_viewport(&UserEvent::Mouse(event::MouseEvent::Exit))?;
            }
        }

        // If the user entered a new viewport, notify the new viewport
        if let Some(curr) = curr_viewport {
            if active_viewport.is_none_or(|active| curr != active) {
                self.active_viewport = Some(curr);
                self.event_context.last_mouse_pos = None;
                self.send_event_to_active_viewport(&UserEvent::Mouse(event::MouseEvent::Enter))?;
            }
        }

        // Always emit the mouse move event to the most-recent viewport
        self.event_context.last_mouse_pos = Some(event::PhysicalPosition { x, y });
        self.send_event_to_active_viewport(&UserEvent::Mouse(event::MouseEvent::Move { x, y }))?;
        Ok(Default::default())
    }

    pub fn handle_mouse_exit(
        &mut self,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        self.send_event_to_active_viewport(&UserEvent::Mouse(event::MouseEvent::Exit))?;
        self.active_viewport = None;
        Ok(Default::default())
    }

    pub fn handle_wheel(
        &mut self,
        dx: f64,
        dy: f64,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        let (dx, dy) = (-dx, -dy);
        self.send_event_to_active_viewport(&UserEvent::Wheel { dx, dy })?;
        Ok(Default::default())
    }

    pub fn handle_modifiers_changed(&mut self, modifiers: u32) {
        self.event_context.modifiers = EventModifierKeys::from_bits_truncate(modifiers);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
