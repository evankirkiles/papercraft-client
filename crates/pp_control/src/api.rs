use crate::App;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl App {
    pub fn hi(&self) -> i32 {
        2
    }
}
