use pp_core::measures::Dimensions;
use pp_editor::tool;
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
    Move(cgmath::Point2<f32>),
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
    MouseWheel { delta: cgmath::Point2<f32> },
}

// Successful responses of an Event Handler
#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum ExternalEventHandleSuccess {
    #[default]
    ContinuePropagation,
    StopPropagation,
}

// All potential errors that can come from event handlers.
#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum ExternalEventHandleError {
    #[default]
    Unknown,
}

impl std::error::Error for ExternalEventHandleError {}
impl core::fmt::Display for ExternalEventHandleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A common event context making core state objects available inside of event
/// handlers, including the state of any modifiers.
#[derive(Debug, Default, Clone)]
pub(crate) struct EventContext {
    pub(crate) state: Rc<RefCell<pp_core::State>>,
    pub(crate) history: Rc<RefCell<pp_core::CommandStack>>,
    pub(crate) renderer: Rc<RefCell<Option<pp_draw::Renderer<'static>>>>,
    pub(crate) modifiers: keyboard::ModifierKeys,
    pub(crate) surface_dpi: f32,
    pub(crate) surface_size: Dimensions<f32>,
    pub(crate) last_mouse_pos: Option<cgmath::Point2<f32>>,
}

impl From<EventHandleSuccess> for ExternalEventHandleSuccess {
    fn from(value: EventHandleSuccess) -> Self {
        value.external
    }
}

impl From<EventHandleError> for ExternalEventHandleError {
    fn from(value: EventHandleError) -> Self {
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
    ) -> Option<Result<EventHandleSuccess, EventHandleError>>;
}

/// This is the internal event success type, which can be intercepted by the
/// top-level controller to use nested information within.
#[derive(Debug, Default, Clone)]
pub struct EventHandleSuccess {
    pub set_tool: Option<Option<tool::Tool>>,
    pub stop_propagation: bool,
    pub external: ExternalEventHandleSuccess,
}

impl EventHandleSuccess {
    pub fn set_tool(tool: Option<tool::Tool>) -> Self {
        Self {
            set_tool: Some(tool),
            stop_propagation: true,
            external: ExternalEventHandleSuccess::StopPropagation,
        }
    }

    pub fn stop_internal_propagation() -> Self {
        Self {
            set_tool: None,
            stop_propagation: true,
            external: ExternalEventHandleSuccess::ContinuePropagation,
        }
    }

    pub fn stop_propagation() -> Self {
        Self {
            set_tool: None,
            stop_propagation: true,
            external: ExternalEventHandleSuccess::StopPropagation,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct EventHandleError {
    pub external: ExternalEventHandleError,
}
