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
    /// changes, receiving the fresh editor snapshot as its argument. Returns an
    /// `unsubscribe` function which de-registers the callback.
    ///
    /// The snapshot is passed directly rather than having the callback call back
    /// into `get_editor_snapshot()`: wasm-bindgen forbids any reentrant call into
    /// a `#[wasm_bindgen]` method on `App` while another call on the same
    /// instance (here, `draw`) is still on the stack ("recursive use of an
    /// object" panic), which is exactly what would happen if the callback tried
    /// to fetch the snapshot itself.
    pub fn on_editor_state_change(&mut self, callback: js_sys::Function) -> js_sys::Function {
        let callbacks = self.callbacks.clone();
        let id = self.callbacks.borrow_mut().editor.insert(callback);
        Closure::once_into_js(move || {
            callbacks.borrow_mut().editor.remove(id);
        })
        .into()
    }

    /// Invokes all registered `on_editor_state_change` callbacks with the given
    /// editor snapshot, notifying JS that the editor state has changed.
    pub(crate) fn fire_editor_callbacks(&self, snapshot: &JsValue) {
        self.callbacks.borrow().editor.values().for_each(|cb| {
            let _ = cb.call1(&JsValue::NULL, snapshot);
        });
    }
}
