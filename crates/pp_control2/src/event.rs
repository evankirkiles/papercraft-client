use bitflags::bitflags;
use paste::paste;
use wasm_bindgen::prelude::*;

pub enum MouseEvent {
    Enter,
    Exit,
    Move { x: f64, y: f64 },
}

pub enum UserEvent {
    Mouse(MouseEvent),
    Wheel { dx: f64, dy: f64 },
}

// Successful responses of an Event Handler
#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy)]
pub enum EventHandleSuccess {
    #[default]
    ContinuePropagation,
    StopPropagation,
}

// All potential errors that can come from event handlers.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum EventHandleError {
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
#[derive(Debug, Default, Clone, Copy)]
pub struct EventContext {
    pub modifiers: EventModifierKeys,
    pub surface_dpi: f64,
    pub surface_size: PhysicalSize<f64>,
    pub last_mouse_pos: Option<PhysicalPosition<f64>>,
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct EventModifierKeys: u32 {
        const LSHIFT = 0b00000001;
        const RSHIFT = 0b00000010;
        const LALT = 0b00000100;
        const RALT = 0b00001000;
        const LCTRL = 0b00010000;
        const RCTRL = 0b00100000;
        const LSUPER = 0b01000000;
        const RSUPER = 0b10000000;
    }
}

macro_rules! pressed_impl {
    ( $key:ident) => {
        paste! {
            pub fn [<$key:lower _pressed>](&self) -> bool {
                self.intersects(EventModifierKeys::$key)
            }
        }
    };
}

impl EventModifierKeys {
    pressed_impl! { LSHIFT }
    pressed_impl! { RSHIFT }
    pressed_impl! { LALT }
    pressed_impl! { RALT }
    pressed_impl! { LCTRL }
    pressed_impl! { RCTRL }
    pressed_impl! { LSUPER }
    pressed_impl! { RSUPER }

    pub fn shift_pressed(&self) -> bool {
        self.lshift_pressed() || self.rshift_pressed()
    }
    pub fn alt_pressed(&self) -> bool {
        self.lalt_pressed() || self.ralt_pressed()
    }
    pub fn ctrl_pressed(&self) -> bool {
        self.lctrl_pressed() || self.rctrl_pressed()
    }
    pub fn super_pressed(&self) -> bool {
        self.lsuper_pressed() || self.rsuper_pressed()
    }
}
