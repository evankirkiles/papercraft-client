use std::{cell::RefCell, rc::Rc};

use crate::event::{self, EventHandler};

#[derive(Debug)]
pub struct Controller2D {
    state: Rc<RefCell<pp_core::State>>,
    renderer: Rc<RefCell<Option<pp_draw::Renderer<'static>>>>,
}

impl Controller2D {
    pub fn new(
        state: Rc<RefCell<pp_core::State>>,
        renderer: Rc<RefCell<Option<pp_draw::Renderer<'static>>>>,
    ) -> Self {
        Self { state, renderer }
    }
}

impl EventHandler for Controller2D {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        // First, call any active tool's event handler. If it returns "true",
        // then do not process the event.

        // If no tool took the event, pass it to the camera.
        match ev {
            // event::UserEvent::Mouse(ev) => match ev {
            //     event::MouseEvent::Enter => todo!(),
            //     event::MouseEvent::Exit => todo!(),
            //     event::MouseEvent::Move { x, y } => todo!(),
            // },
            event::UserEvent::Wheel { dx, dy } => {
                let mut state = self.state.borrow_mut();
                if ctx.modifiers.alt_pressed() {
                    state.viewport_2d.camera.dolly(*dy * 0.5);
                } else {
                    state.viewport_2d.camera.pan(*dx, *dy);
                }
            }
            _ => {}
        };
        Ok(event::EventHandleSuccess::StopPropagation)
    }
}
