use ascii::AsciiChar;
use bitflags::bitflags;
use compact_str::CompactString;
use paste::paste;
use wasm_bindgen::prelude::*;

bitflags! {
    /// Meta-keys which augment the functionality of other keys when pressed simultaneously
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ModifierKeys: u32 {
        const LSHIFT = 0b000_00001;
        const RSHIFT = 0b000_00010;
        const LALT = 0b0000_0100;
        const RALT = 0b0000_1000;
        const LCTRL = 0b0001_0000;
        const RCTRL = 0b0010_0000;
        const LSUPER = 0b0100_0000;
        const RSUPER = 0b1000_0000;
    }
}

macro_rules! pressed_impl {
    ( $key:ident) => {
        paste! {
            pub fn [<$key:lower _pressed>](&self) -> bool {
                self.intersects(ModifierKeys::$key)
            }
        }
    };
}

impl ModifierKeys {
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

/// Meta-keys which do not map to an exact typed character.
#[wasm_bindgen]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NamedKey {
    Alt,
    CapsLock,
    Control,
    Enter,
    Meta,
    Redo,
    Tab,
    Undo,
    Escape,
}

/// A key pressed by a user. Corresponds to `event.key`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Key {
    Named(NamedKey),
    /// An ASCII character typed by the user. While it'd be nice to support
    /// locales, the performance overhead of passing strings to WASM is not nice.
    Character(AsciiChar),
}

impl From<NamedKey> for Key {
    #[inline]
    fn from(action: NamedKey) -> Self {
        Key::Named(action)
    }
}
