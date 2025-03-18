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
pub enum Key<Str = CompactString> {
    Named(NamedKey),
    /// An ASCII character typed by the user. While it'd be nice to support
    /// locales, the performance overhead of passing strings to WASM is not nice.
    Character(Str),
}

impl From<NamedKey> for Key {
    #[inline]
    fn from(action: NamedKey) -> Self {
        Key::Named(action)
    }
}

impl<Str> PartialEq<NamedKey> for Key<Str> {
    #[inline]
    fn eq(&self, rhs: &NamedKey) -> bool {
        match self {
            Key::Named(ref a) => a == rhs,
            _ => false,
        }
    }
}

impl<Str: PartialEq<str>> PartialEq<str> for Key<Str> {
    #[inline]
    fn eq(&self, rhs: &str) -> bool {
        match self {
            Key::Character(ref s) => s == rhs,
            _ => false,
        }
    }
}

impl<Str: PartialEq<str>> PartialEq<&str> for Key<Str> {
    #[inline]
    fn eq(&self, rhs: &&str) -> bool {
        self == *rhs
    }
}
impl Key<CompactString> {
    /// Convert `Key::Character(SmolStr)` to `Key::Character(&str)` so you can more easily match on
    /// `Key`. All other variants remain unchanged.
    pub fn as_ref(&self) -> Key<&str> {
        match self {
            Key::Named(a) => Key::Named(*a),
            Key::Character(ch) => Key::Character(ch.as_str()),
        }
    }
}

impl Key {
    pub fn from_key_code(kav: &str) -> Self {
        Key::Named(match kav {
            "Alt" => NamedKey::Alt,
            "CapsLock" => NamedKey::CapsLock,
            "Control" => NamedKey::Control,
            "Enter" => NamedKey::Enter,
            "Meta" => NamedKey::Meta,
            "Redo" => NamedKey::Redo,
            "Tab" => NamedKey::Tab,
            "Undo" => NamedKey::Undo,
            "Escape" => NamedKey::Escape,
            string => return Key::Character(CompactString::new(string)),
        })
    }
}
