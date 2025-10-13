use event::{
    EventContext, EventHandleSuccess, EventHandler, ExternalEventHandleError,
    ExternalEventHandleSuccess, PressedState, UserEvent,
};
use keyboard::ModifierKeys;
use pp_core::measures::Dimensions;
use pp_editor::SplitId;
use pp_save::{load::Loadable, SaveFile};
use slotmap::KeyData;
use std::{cell::RefCell, io::Cursor, ops::DerefMut, rc::Rc};
use store::AppCallbacks;
use wasm_bindgen::prelude::*;

mod editor;
mod event;
mod keyboard;
mod store;
mod tool;
mod viewport;

#[wasm_bindgen(typescript_custom_section)]
const SLOTMAP_TYPES: &'static str = r#"
export type KeyData = { idx: number; version: number };
export type SlotMap<T, U> = { value: U, version: number }[];
"#;

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct App {
    /// The core model of the app, synchronized with the server
    state: Rc<RefCell<pp_core::State>>,
    /// The command stack for undoing / redoing operations
    history: Rc<RefCell<pp_core::CommandStack>>,
    /// The GPU resources of the App. Only created once a canvas is `attach`ed.
    renderer: Rc<RefCell<Option<pp_draw::Renderer<'static>>>>,
    /// The client-side state of the app
    editor: pp_editor::Editor,

    /// Callbacks for synchronizing internal state with React state
    callbacks: Rc<RefCell<AppCallbacks>>,

    /// A common event context used across all event handlers
    event_context: EventContext,
}

/// "App" holds the entirey of the Rust application state. You can think of it
/// as the controller owning the Model (`pp_core`) and the View (`pp_draw`).
#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let state = Rc::new(RefCell::new(pp_core::State::default()));
        let history = Rc::new(RefCell::new(pp_core::CommandStack::default()));
        let renderer = Rc::new(RefCell::<Option<pp_draw::Renderer<'static>>>::new(None));
        Self {
            event_context: EventContext {
                state: state.clone(),
                history: history.clone(),
                renderer: renderer.clone(),
                ..Default::default()
            },
            state,
            history,
            renderer,
            ..Default::default()
        }
    }

    /// Reloads the app from the uncompressed bytes of a save file. Note that
    /// the `renderer`'s resources are untouched - these will be automatically
    /// synchronized with the new `state`.
    pub fn load_save(&mut self, bytes: &[u8]) -> Result<(), JsError> {
        let save_file = SaveFile::from_reader(Cursor::new(bytes))
            .map_err(|_| JsError::new("Failed to load save file."))?;
        let state = pp_core::State::load(save_file)?;
        self.state.replace(state);
        self.history.replace(pp_core::CommandStack::default());
        self.editor.reset();
        Ok(())
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

    /// De-allocates all the GPU resources for the app
    pub fn unattach(&mut self) {
        self.renderer.replace(None);
    }

    /// Returns a snapshot of the editor's state
    pub fn get_editor_snapshot(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.editor)?)
    }

    // ---- RENDER CYCLE -----
    // Functions called in a loop or in a global listener, relevant to the renderer.

    /// Updates the internal state of any time-based states in the canvas, e.g.
    /// scene changes which aren't caused directly by an interaction (like animations)
    pub fn update(&mut self, _timestamp: u32) -> Result<(), JsError> {
        let mut renderer = self.renderer.borrow_mut();
        let renderer = renderer.as_mut().ok_or(AppError::NoCanvasAttached)?;
        renderer.select_poll();
        Ok(())
    }

    /// Draws a single frame of the app to the canvas.
    pub fn draw(&mut self, _timestamp: u32) -> Result<(), JsError> {
        let mut renderer = self.renderer.borrow_mut();
        let mut state = self.state.borrow_mut();
        let renderer = renderer.as_mut().ok_or(AppError::NoCanvasAttached)?;
        let state = state.deref_mut();
        renderer.prepare(state, &mut self.editor);
        renderer.render(state);
        Ok(())
    }

    /// Resizes the virtual dimensions of the canvas.
    pub fn resize(&mut self, width: f32, height: f32, dpr: f32) -> Result<(), JsError> {
        let dimensions = Dimensions { width: width * dpr, height: height * dpr };
        let mut renderer = self.renderer.borrow_mut();
        let renderer = renderer.as_mut().ok_or(AppError::NoCanvasAttached)?;
        renderer.resize(&dimensions.into());
        self.editor.resize(&dimensions, dpr);
        // TODO: Remove this other PhysicalDimensions type
        self.event_context.surface_size = Dimensions { width, height };
        self.event_context.surface_dpi = dpr;
        Ok(())
    }

    // ---- HOOKS ----
    // Functions that can be invoked by JavaScript on user interaction with HTML.

    /// Updates the split ratio between two viewports
    pub fn update_split(&mut self, id: u64, ratio: f32) {
        let id: SplitId = KeyData::from_ffi(id).into();
        if let Some(split) = self.editor.splits.get_mut(id) {
            split.ratio = ratio;
            split.is_dirty = true;
            self.editor.update();
        }
    }

    /// Sets the select mode of the application
    pub fn set_select_mode(&mut self, select_mode: pp_core::settings::SelectionMode) {
        let mut state = self.state.borrow_mut();
        state.settings.selection_mode = select_mode;
    }

    /// Internal function used to route an event to the viewport a user is currently
    /// interacting with, e.g. where their mouse is hovered. If the event still
    /// propagated, then the controller can maybe do some last-minute processing.
    fn handle_event(
        &mut self,
        ev: &UserEvent,
    ) -> Result<ExternalEventHandleSuccess, ExternalEventHandleError> {
        // 1. If a tool is active, it gets all input until canceled
        let res =
            self.editor.active_tool.as_mut().and_then(|t| t.handle_event(&self.event_context, ev));
        if let Some(result) = res.and_then(|res| self.process_event(res)) {
            return result;
        }

        // 2. Otherwise, pass input into the viewport
        let viewport = self.editor.active_viewport.and_then(|v| self.editor.viewports.get_mut(v));
        let res = viewport.and_then(|viewport| viewport.handle_event(&self.event_context, ev));
        if let Some(result) = res.and_then(|res| self.process_event(res)) {
            return result;
        }

        // 3. If no viewport-specific functionality, pass to the editor itself
        let res = self.editor.handle_event(&self.event_context, ev);
        if let Some(result) = res.and_then(|res| self.process_event(res)) {
            return result;
        }

        Ok(ExternalEventHandleSuccess::default())
    }

    /// Applies any side-effects from an internal event handler, mapping the
    /// result back into the top-level type.
    fn process_event(
        &mut self,
        res: Result<EventHandleSuccess, event::EventHandleError>,
    ) -> Option<Result<event::ExternalEventHandleSuccess, event::ExternalEventHandleError>> {
        match res {
            Ok(res) => {
                // Apply any active tool passed from the handler
                if let Some(active_tool) = res.set_tool {
                    self.editor.active_tool = active_tool;
                }
                res.stop_propagation.then_some(Ok(res.external))
            }
            Err(_) => Some(Err(ExternalEventHandleError::Unknown)),
        }
    }

    // ---- HANDLERS -----
    // Functions that are invoked directly by a JavaScript listener.

    pub fn handle_mouse_enter(
        &mut self,
        x: f32,
        y: f32,
    ) -> Result<event::ExternalEventHandleSuccess, event::ExternalEventHandleError> {
        let pos = cgmath::Point2::new(x, y);
        self.editor.active_viewport = self.editor.viewport_at(pos * self.editor.dpr);
        self.event_context.last_mouse_pos = None;
        self.handle_event(&UserEvent::Pointer(event::PointerEvent::Enter))
    }

    pub fn handle_mouse_move(
        &mut self,
        x: f32,
        y: f32,
    ) -> Result<event::ExternalEventHandleSuccess, event::ExternalEventHandleError> {
        let pos = cgmath::Point2::new(x, y);
        let curr_viewport = self.editor.viewport_at(pos * self.editor.dpr);

        // If the user left an active viewport, notify the old viewport
        if let Some(active) = self.editor.active_viewport {
            if curr_viewport.is_none_or(|curr| curr != active) {
                self.handle_event(&UserEvent::Pointer(event::PointerEvent::Exit))?;
            }
        }

        // If the user entered a new viewport, notify the new viewport
        if let Some(curr) = curr_viewport {
            if self.editor.active_viewport.is_none_or(|active| curr != active) {
                self.editor.active_viewport = Some(curr);
                self.event_context.last_mouse_pos = None;
                self.handle_event(&UserEvent::Pointer(event::PointerEvent::Enter))?;
            }
        }

        // Always emit the mouse move event to the most-recent viewport
        let pos = cgmath::Point2::new(x, y);
        self.event_context.last_mouse_pos = Some(pos);
        self.handle_event(&UserEvent::Pointer(event::PointerEvent::Move(pos)))
    }

    pub fn handle_mouse_exit(
        &mut self,
    ) -> Result<event::ExternalEventHandleSuccess, event::ExternalEventHandleError> {
        let res = self.handle_event(&UserEvent::Pointer(event::PointerEvent::Exit));
        self.editor.active_viewport = None;
        res
    }

    pub fn handle_wheel(
        &mut self,
        dx: f32,
        dy: f32,
    ) -> Result<event::ExternalEventHandleSuccess, event::ExternalEventHandleError> {
        self.handle_event(&UserEvent::MouseWheel { delta: cgmath::Point2::new(-dx, -dy) })
    }

    pub fn handle_modifiers_changed(&mut self, modifiers: u32) {
        self.event_context.modifiers = ModifierKeys::from_bits_truncate(modifiers);
    }

    /// Handles named key input.
    pub fn handle_named_key(
        &mut self,
        key: keyboard::NamedKey,
        pressed: PressedState,
    ) -> Result<event::ExternalEventHandleSuccess, event::ExternalEventHandleError> {
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
    ) -> Result<event::ExternalEventHandleSuccess, event::ExternalEventHandleError> {
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
    ) -> Result<event::ExternalEventHandleSuccess, event::ExternalEventHandleError> {
        self.handle_event(&UserEvent::MouseInput(match pressed {
            PressedState::Pressed => event::MouseInputEvent::Down(button),
            PressedState::Unpressed => event::MouseInputEvent::Up(button),
        }))
    }
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

/// Instruments Rust's logger with `console.log` capabilities on the web.
/// Call this once and only once at the start of the application.
#[wasm_bindgen]
pub fn install_logging() {
    // Set up console logging / console error
    // #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
}
