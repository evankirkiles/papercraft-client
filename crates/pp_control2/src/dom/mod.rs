use std::{fmt::Debug, ops::Deref, rc::Rc};

use event::EventListenerHandle;
use wasm_bindgen::prelude::*;
use web_sys::{PointerEvent, WheelEvent};

mod event;

#[derive(Debug)]
pub struct DOMBindings {
    pub window: DOMWindow,
    pub canvas: DOMCanvas,
}

impl DOMBindings {
    pub fn new(canvas_id: &str) -> Self {
        let window = DOMWindow::new();
        let canvas = DOMCanvas::find(&window.handle, canvas_id).unwrap();
        Self { window, canvas }
    }
}

pub struct DOMWindow {
    pub handle: Rc<web_sys::Window>,
    on_resize: Option<EventListenerHandle<dyn FnMut(web_sys::Event)>>,
}

impl std::fmt::Debug for DOMWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DOMWindow").finish()
    }
}

impl DOMWindow {
    pub fn new() -> Self {
        let window = web_sys::window().expect("Missing window");
        Self { handle: Rc::new(window), on_resize: None }
    }

    pub fn on_resize<F>(&mut self, handler: F)
    where
        F: 'static + FnMut(web_sys::Event),
    {
        self.on_resize = Some(EventListenerHandle::new(
            self.handle.deref().clone(),
            "resize",
            Closure::new(handler),
        ))
    }
}

#[derive(Debug)]
pub enum DOMCanvasFindError {
    MissingDocument,
    MissingCanvas,
}

pub struct DOMCanvas {
    pub handle: Rc<web_sys::HtmlCanvasElement>,
    on_wheel: Option<EventListenerHandle<dyn FnMut(web_sys::WheelEvent)>>,
    on_pointer_move: Option<EventListenerHandle<dyn FnMut(web_sys::PointerEvent)>>,
    on_pointer_enter: Option<EventListenerHandle<dyn FnMut(web_sys::PointerEvent)>>,
    on_pointer_exit: Option<EventListenerHandle<dyn FnMut(web_sys::PointerEvent)>>,
}

impl std::fmt::Debug for DOMCanvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DOMCanvas").finish()
    }
}

impl DOMCanvas {
    pub fn find(window: &web_sys::Window, id: &str) -> Result<Self, DOMCanvasFindError> {
        let document = window.document().ok_or(DOMCanvasFindError::MissingDocument)?;
        let canvas = document
            .get_element_by_id(id)
            .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .ok_or(DOMCanvasFindError::MissingCanvas)?;
        Ok(Self {
            handle: Rc::new(canvas),
            on_wheel: None,
            on_pointer_move: None,
            on_pointer_enter: None,
            on_pointer_exit: None,
        })
    }

    pub fn on_wheel<C>(&mut self, handler: C)
    where
        C: 'static + FnMut(WheelEvent),
    {
        self.on_wheel = Some(EventListenerHandle::new(
            self.handle.deref().clone(),
            "wheel",
            Closure::new(handler),
        ))
    }

    pub fn on_pointer_move<C>(&mut self, handler: C)
    where
        C: 'static + FnMut(PointerEvent),
    {
        self.on_pointer_move = Some(EventListenerHandle::new(
            self.handle.deref().clone(),
            "pointermove",
            Closure::new(handler),
        ))
    }
}
