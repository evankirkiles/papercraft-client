use editor::EditorEventHandler;
use event::{
    EventContext, EventHandleSuccess, EventHandler, ExternalEventHandleError,
    ExternalEventHandleSuccess, PressedState, UserEvent,
};
use keyboard::ModifierKeys;
use pp_core::measures::Dimensions;
use pp_editor::SplitId;
use pp_save::{load::Loadable, SaveFile};
use serde::{Deserialize, Serialize};
use slotmap::KeyData;
use std::{cell::RefCell, io::Cursor, ops::DerefMut, rc::Rc};
use store::AppCallbacks;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::command::sync::SyncConnectionConfig;

mod command;
mod editor;
mod event;
mod keyboard;
mod print;
mod store;
mod tool;
mod viewport;

#[wasm_bindgen(typescript_custom_section)]
const SLOTMAP_TYPES: &'static str = r#"
export type KeyData = { idx: number; version: number };
export type SlotMap<T, U> = { value: U, version: number }[];
"#;

/// The real-world dimensions of the document's bounding box, in centimeters.
#[derive(Debug, Default, Clone, Copy, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct MeshBoundsCm {
    pub width: f32,
    pub depth: f32,
    pub height: f32,
}

/// The largest frame delta time-based state will act on, so a tab that stops
/// rendering (backgrounded, or blocked on a long load) resumes smoothly instead
/// of jumping by however long it was away.
const MAX_FRAME_DELTA_MS: f32 = 100.0;

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct App {
    /// The core model of the app, synchronized with the server
    state: Rc<RefCell<pp_core::State>>,
    /// The command stack for undoing / redoing operations
    history: Rc<RefCell<command::MultiplayerCommandStack>>,
    /// The GPU resources of the App. Only created once a canvas is `attach`ed.
    renderer: Rc<RefCell<Option<pp_draw::Renderer<'static>>>>,
    /// The client-side state of the app
    editor: pp_editor::Editor,

    /// Callbacks for synchronizing internal state with React state
    callbacks: Rc<RefCell<AppCallbacks>>,

    /// A common event context used across all event handlers
    event_context: EventContext,

    /// The `requestAnimationFrame` timestamp of the last `update`, used to
    /// derive the frame delta that drives time-based state like camera moves.
    last_timestamp: Option<u32>,

    /// The print run in flight, advanced one page per `update`. At most one
    /// runs at a time - they share a readback buffer, and the archive they
    /// produce is per-run.
    print_job: Option<print::PrintJob>,
}

/// "App" holds the entirey of the Rust application state. You can think of it
/// as the controller owning the Model (`pp_core`) and the View (`pp_draw`).
#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let state = Rc::new(RefCell::new(pp_core::State::default()));
        let history = Rc::new(RefCell::new(command::MultiplayerCommandStack::default()));
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
            callbacks: Rc::new(RefCell::new(AppCallbacks::default())),
            editor: pp_editor::Editor::default(),
            last_timestamp: None,
            print_job: None,
        }
    }

    /// Reloads the app by connecting to a `pp_server` websocket, where we recieve
    /// the save file bytes and live-connect to the websocket server for additional changes.
    pub fn load_live(&mut self, config: SyncConnectionConfig) -> Result<(), JsValue> {
        // Reset history to default state
        self.history.take();
        self.editor.reset();
        // Correct state will be streamed in over websocket
        self.history.borrow_mut().subscribe(self.state.clone(), &config)?;
        Ok(())
    }

    /// Reloads the app from the uncompressed bytes of a save file. Note that
    /// the `renderer`'s resources are untouched - these will be automatically
    /// synchronized with the new `state`.
    pub fn load_save(&mut self, bytes: &[u8]) -> Result<(), JsError> {
        let save_file = SaveFile::from_reader(Cursor::new(bytes))
            .map_err(|_| JsError::new("Failed to load save file."))?;
        let state = pp_core::State::load(save_file)?;
        self.state.replace(state);
        self.history.take();
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
        // A print run lives on the GPU we're about to drop, so it can never
        // finish. Settle its promise rather than leaving the caller hanging.
        if let Some(job) = self.print_job.take() {
            job.abort("The canvas was detached before printing finished");
        }
        self.renderer.replace(None);
    }

    /// Renders every page of the print layout and downloads them as one PDF.
    ///
    /// Each page is the cutting viewport's own render of that sheet - textured
    /// surfaces, fold and cut lines, and tabs - rasterized at `dpi` (300 by
    /// default) and placed into the PDF as an image.
    ///
    /// Pages render one per frame, so the returned promise settles some frames
    /// later: with the number of pages written, or with the reason the run
    /// failed.
    pub fn print(&mut self, dpi: Option<f32>) -> js_sys::Promise {
        let mut callbacks = None;
        // `Promise::new` runs its executor synchronously, so the functions are
        // always there by the time we look.
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            callbacks = Some((resolve, reject));
        });
        let (resolve, reject) = callbacks.expect("Promise executor did not run");
        if let Err(err) = self.start_print(
            dpi.unwrap_or(pp_draw::print::DEFAULT_PRINT_DPI),
            resolve,
            reject.clone(),
        ) {
            let _ = reject.call1(&JsValue::NULL, &err);
        }
        promise
    }

    fn start_print(
        &mut self,
        dpi: f32,
        resolve: js_sys::Function,
        reject: js_sys::Function,
    ) -> Result<(), JsValue> {
        if self.print_job.is_some() {
            return Err(JsValue::from_str("A print run is already in progress"));
        }
        let renderer = self.renderer.borrow();
        let renderer = renderer.as_ref().ok_or(JsValue::from_str("No canvas is attached"))?;

        // The grid is normally fitted in `draw`, but printing before the first
        // frame after an edit would otherwise miss a page.
        let mut state = self.state.borrow_mut();
        state.fit_pages_to_pieces();
        let target = pp_draw::print::PrintTarget::new(
            &renderer.ctx,
            &state.printing.page_size,
            &self.editor.preferences.theme,
            dpi,
        )
        .map_err(|err| JsValue::from_str(&err.to_string()))?;

        let pages = print::pages_to_render(&state);
        self.print_job = Some(print::PrintJob::new(
            target,
            state.printing.page_size.dimensions(),
            pages,
            resolve,
            reject,
        ));
        Ok(())
    }

    /// Returns a snapshot of the editor's state
    pub fn get_editor_snapshot(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.editor)?)
    }

    // ---- RENDER CYCLE -----
    // Functions called in a loop or in a global listener, relevant to the renderer.

    /// Updates the internal state of any time-based states in the canvas, e.g.
    /// scene changes which aren't caused directly by an interaction (like animations)
    pub fn update(&mut self, timestamp: u32) -> Result<(), JsError> {
        // Advance any camera animations before the renderer is needed, so they
        // keep running even in the frames before a canvas is attached. The
        // delta is capped so a backgrounded tab, which stops firing frames,
        // doesn't resume by teleporting the camera most of the way there.
        let dt_ms = self
            .last_timestamp
            .map(|last| (timestamp.wrapping_sub(last) as f32).clamp(0.0, MAX_FRAME_DELTA_MS))
            .unwrap_or(0.0);
        self.last_timestamp = Some(timestamp);
        self.editor.tick_cameras(dt_ms);

        let mut renderer = self.renderer.borrow_mut();
        let renderer = renderer.as_mut().ok_or(AppError::NoCanvasAttached)?;
        renderer.select_poll();
        if self.print_job.as_mut().is_some_and(|job| !job.tick(renderer)) {
            self.print_job = None;
        }
        if self.editor.is_dirty {
            self.editor.is_dirty = false;
            let snapshot = serde_wasm_bindgen::to_value(&self.editor)?;
            self.fire_editor_callbacks(&snapshot);
        }
        Ok(())
    }

    /// Draws a single frame of the app to the canvas.
    pub fn draw(&mut self, _timestamp: u32) -> Result<(), JsError> {
        let mut renderer = self.renderer.borrow_mut();
        let mut state = self.state.borrow_mut();
        let renderer = renderer.as_mut().ok_or(AppError::NoCanvasAttached)?;
        let state = state.deref_mut();
        state.fit_pages_to_pieces();
        renderer.prepare(state, &mut self.editor);
        renderer.render(state, &self.editor);
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
        if let Some(split) = self.editor.layout.splits.get_mut(id) {
            split.ratio = ratio;
            split.is_dirty = true;
            self.editor.update();
        }
    }

    /// Sets the select mode of the application
    pub fn set_select_mode(&mut self, select_mode: pp_editor::state::SelectionMode) {
        self.editor.state.selection_mode = select_mode;
        self.editor.is_dirty = true;
        // A cached select buffer was rendered for the old mask, so the paint
        // tool has to re-capture before it can pick up the new element type.
        if matches!(self.editor.active_tool, Some(pp_editor::tool::Tool::SelectPaint(_))) {
            tool::select_paint::capture_select_buffer(&self.event_context, &self.editor.state);
        }
    }

    /// Switches between the box-drag and brush-paint selection gestures, the
    /// JS-side equivalent of the `C` / `Esc` keybinds.
    pub fn set_select_tool(&mut self, select_tool: pp_editor::state::SelectTool) {
        use pp_editor::{
            state::SelectTool,
            tool::{select_paint::DEFAULT_RADIUS, SelectPaintTool, Tool},
        };
        if self.editor.state.select_tool == select_tool {
            return;
        }
        self.editor.state.select_tool = select_tool;
        self.editor.is_dirty = true;
        match select_tool {
            SelectTool::Paint => {
                let cursor_pos =
                    self.event_context.last_mouse_pos.unwrap_or(cgmath::Point2::new(0.0, 0.0))
                        * self.event_context.surface_dpi;
                let tool = SelectPaintTool::new(
                    cursor_pos,
                    DEFAULT_RADIUS * self.event_context.surface_dpi,
                );
                tool::select_paint::capture_select_buffer(&self.event_context, &self.editor.state);
                self.editor.active_tool = Some(Tool::SelectPaint(tool));
            }
            SelectTool::Box => {
                if matches!(self.editor.active_tool, Some(Tool::SelectPaint(_))) {
                    self.editor.active_tool = None;
                }
            }
        }
    }

    /// Sets the x-ray mode of the application
    pub fn set_is_xray(&mut self, is_xray: bool) {
        self.editor.state.is_xray = is_xray;
        self.editor.is_dirty = true;
    }

    /// Applies an incremental uniform scale factor to every mesh in the
    /// document, e.g. `new_scale / old_scale` computed by the caller.
    pub fn scale_mesh(&mut self, factor: f32) {
        let meshes = self.state.borrow().meshes.keys().collect();
        let cmd =
            pp_core::CommandType::ScaleMesh(pp_core::commands::scale_mesh::ScaleMeshCommand {
                meshes,
                factor,
            });
        self.history
            .borrow_mut()
            .execute(&mut self.state.borrow_mut(), cmd)
            .expect("scale_mesh command should never fail");
    }

    /// Returns the real-world dimensions of the document's world-space
    /// bounding box, in centimeters (1 world unit = 1 cm). All-zero if there
    /// are no meshes / vertices. Unit formatting (cm vs. m) is left to JS.
    pub fn get_mesh_bounds(&self) -> MeshBoundsCm {
        let aabb = self.state.borrow().world_bounds();
        if aabb.is_empty() {
            return MeshBoundsCm::default();
        }
        let size = aabb.size();
        MeshBoundsCm { width: size.x, depth: size.y, height: size.z }
    }

    /// Internal function used to route an event to the viewport a user is currently
    /// interacting with, e.g. where their mouse is hovered. If the event still
    /// propagated, then the controller can maybe do some last-minute processing.
    fn handle_event(
        &mut self,
        ev: &UserEvent,
    ) -> Result<ExternalEventHandleSuccess, ExternalEventHandleError> {
        // 1. If a tool is active, it gets all input until canceled
        let res = match self.editor.active_tool.as_mut() {
            Some(t) => t.handle_event(&self.event_context, &mut self.editor.state, ev),
            None => None,
        };
        if let Some(result) = res.and_then(|res| self.process_event(res)) {
            return result;
        }

        // 2. Otherwise, pass input into the viewport
        let viewport =
            self.editor.active_viewport.and_then(|v| self.editor.layout.viewports.get_mut(v));
        let res = match viewport {
            Some(viewport) => {
                viewport.handle_event(&self.event_context, &mut self.editor.state, ev)
            }
            None => None,
        };
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
                    self.editor.is_dirty = true;
                }
                // Push a snapshot for handlers which mutated the editor state
                if res.mark_dirty {
                    self.editor.is_dirty = true;
                }
                // Apply any x-ray toggle passed from the handler
                if res.toggle_xray {
                    self.editor.state.is_xray = !self.editor.state.is_xray;
                    self.editor.is_dirty = true;
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
        self.editor.active_viewport = self.editor.viewport_at(pos * self.editor.layout.dpr);
        self.event_context.last_mouse_pos = None;
        self.handle_event(&UserEvent::Pointer(event::PointerEvent::Enter))
    }

    pub fn handle_mouse_move(
        &mut self,
        x: f32,
        y: f32,
    ) -> Result<event::ExternalEventHandleSuccess, event::ExternalEventHandleError> {
        let pos = cgmath::Point2::new(x, y);
        let curr_viewport = self.editor.viewport_at(pos * self.editor.layout.dpr);

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
