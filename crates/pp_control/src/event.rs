use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::prelude::*;

use crate::keyboard;

#[derive(Debug, Clone, Copy)]
pub enum PointerEvent {
    Enter,
    Exit,
    Move(PhysicalPosition<f64>),
}

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

/// An mouse button press has been received.
#[derive(Debug, Clone, Copy)]
pub enum MouseInputEvent {
    Down(MouseButton),
    Up(MouseButton),
}

/// An keyboard button press has been received.
#[derive(Debug, Clone)]
pub enum KeyboardInputEvent {
    Down(keyboard::Key),
    Up(keyboard::Key),
}

#[derive(Debug, Clone)]
pub enum UserEvent {
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

/// A trait any event handler can conform to.
pub trait EventHandler {
    /// A basic event handling function. Returns `true` if the event should
    /// not be propagated further, else returns `false`.
    fn handle_event(
        &mut self,
        ctx: &EventContext,
        ev: &UserEvent,
    ) -> Result<EventHandleSuccess, EventHandleError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PhysicalSize<T> {
    pub width: T,
    pub height: T,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PhysicalPosition<T> {
    pub x: T,
    pub y: T,
}

/// A common "event" context, including the state of any modifiers.
#[derive(Debug, Default, Clone)]
pub struct EventContext {
    pub state: Rc<RefCell<pp_core::State>>,
    pub renderer: Rc<RefCell<Option<pp_draw::Renderer<'static>>>>,
    pub modifiers: keyboard::ModifierKeys,
    pub surface_dpi: f64,
    pub surface_size: PhysicalSize<f64>,
    pub last_mouse_pos: Option<PhysicalPosition<f64>>,
}
