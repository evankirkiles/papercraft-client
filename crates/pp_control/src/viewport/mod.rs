use pp_editor::viewport::{Viewport, ViewportBounds};

use crate::{
    event::{self, EventHandleError, EventHandleSuccess},
    keyboard, EventContext, EventHandler, UserEvent,
};

pub mod cutting;
pub mod folding;

/// A trait any event handler can conform to.
pub(crate) trait ViewportEventHandler {
    /// A basic event handling function.
    fn handle_event(
        &mut self,
        ctx: &EventContext,
        ev: &UserEvent,
        bounds: &ViewportBounds,
    ) -> Option<Result<EventHandleSuccess, EventHandleError>>;
}

impl EventHandler for Viewport {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        ev: &crate::UserEvent,
    ) -> Option<Result<crate::event::EventHandleSuccess, crate::event::EventHandleError>> {
        use pp_editor::viewport::ViewportContent;
        let res = match &mut self.content {
            ViewportContent::Folding(vp) => vp.handle_event(ctx, ev, &self.bounds),
            ViewportContent::Cutting(vp) => vp.handle_event(ctx, ev, &self.bounds),
        };
        if res.is_some() {
            return res;
        }

        // Common event handling for ALL viewports
        match ev {
            UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(
                keyboard::Key::Character(char),
            )) => match char.as_str() {
                // W: Toggle preview mode
                "KeyW" => {}
                _ => (),
            },
            _ => (),
        }
        None
    }
}
