use slotmap::{new_key_type, SlotMap};
use wasm_bindgen::prelude::*;
use web_sys::js_sys;

use crate::App;

new_key_type! {
    pub struct CallbackId;
}

#[derive(Debug, Default)]
pub struct AppCallbacks {
    pub editor: SlotMap<CallbackId, js_sys::Function>,
}

#[wasm_bindgen]
impl App {
    /// Registers a callback which is invoked any time the Editor's internal state
    /// changes, allowing UI frameworks to know when to re-fetch the editor state.
    /// Returns an `unsubscribe` function which de-registers the callback.
    pub fn on_editor_state_change(&mut self, callback: js_sys::Function) -> js_sys::Function {
        let callbacks = self.callbacks.clone();
        let id = self.callbacks.borrow_mut().editor.insert(callback);
        Closure::once_into_js(move || {
            callbacks.borrow_mut().editor.remove(id);
        })
        .into()
    }
}
