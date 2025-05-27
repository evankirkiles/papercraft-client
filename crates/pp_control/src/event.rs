use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

use crate::keyboard;

/// Whether or not a button is pressed.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum PressedState {
    Pressed,
    Unpressed,
}

/// Describes a button of a mouse controller.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PointerEvent {
    Enter,
    Exit,
    Move(PhysicalPosition<f64>),
}

/// An mouse button press has been received.
#[derive(Debug, Clone, Copy)]
pub(crate) enum MouseInputEvent {
    Down(MouseButton),
    Up(MouseButton),
}

/// An keyboard button press has been received.
#[derive(Debug, Clone)]
pub(crate) enum KeyboardInputEvent {
    Down(keyboard::Key),
    Up(keyboard::Key),
}

#[derive(Debug, Clone)]
pub(crate) enum UserEvent {
    Pointer(PointerEvent),
    MouseInput(MouseInputEvent),
    KeyboardInput(KeyboardInputEvent),
    MouseWheel { dx: f64, dy: f64 },
}

// Successful responses of an Event Handler
#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum EventHandleSuccess {
    #[default]
    ContinuePropagation,
    StopPropagation,
}

// All potential errors that can come from event handlers.
#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum EventHandleError {
    #[default]
    Unknown,
}

impl std::error::Error for EventHandleError {}
impl core::fmt::Display for EventHandleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct PhysicalSize<T> {
    pub(crate) width: T,
    pub(crate) height: T,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct PhysicalPosition<T> {
    pub(crate) x: T,
    pub(crate) y: T,
}

/// A common "event" context, including the state of any modifiers.
#[derive(Debug, Default, Clone)]
pub(crate) struct EventContext {
    pub(crate) state: Rc<RefCell<pp_core::State>>,
    pub(crate) renderer: Rc<RefCell<Option<pp_draw::Renderer<'static>>>>,
    pub(crate) modifiers: keyboard::ModifierKeys,
    pub(crate) surface_dpi: f64,
    pub(crate) surface_size: PhysicalSize<f64>,
    pub(crate) last_mouse_pos: Option<PhysicalPosition<f64>>,
}

impl From<InternalEventHandleSuccess> for EventHandleSuccess {
    fn from(value: InternalEventHandleSuccess) -> Self {
        value.external
    }
}

impl From<InternalEventHandleError> for EventHandleError {
    fn from(value: InternalEventHandleError) -> Self {
        value.external
    }
}

/// A trait any event handler can conform to.
pub(crate) trait EventHandler {
    /// A basic event handling function.
    fn handle_event(
        &mut self,
        ctx: &EventContext,
        ev: &UserEvent,
    ) -> Result<InternalEventHandleSuccess, InternalEventHandleError>;
}

/// This is the internal event success type, which can be intercepted by the
/// top-level controller to use nested information within.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct InternalEventHandleSuccess {
    pub clear_tool: bool,
    pub stop_propagation: bool,
    pub external: EventHandleSuccess,
}

impl InternalEventHandleSuccess {
    pub fn clear_tool() -> Self {
        Self {
            clear_tool: true,
            stop_propagation: true,
            external: EventHandleSuccess::StopPropagation,
        }
    }

    pub fn stop_internal_propagation() -> Self {
        Self {
            clear_tool: false,
            stop_propagation: true,
            external: EventHandleSuccess::ContinuePropagation,
        }
    }

    pub fn stop_propagation() -> Self {
        Self {
            clear_tool: false,
            stop_propagation: true,
            external: EventHandleSuccess::StopPropagation,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct InternalEventHandleError {
    pub external: EventHandleError,
}

pub type InternalEventHandleResult = Result<InternalEventHandleSuccess, InternalEventHandleError>;
pub type EventHandleResult = Result<EventHandleSuccess, EventHandleError>;
