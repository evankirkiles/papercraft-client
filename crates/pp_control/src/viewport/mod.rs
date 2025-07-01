use pp_editor::viewport::{Viewport, ViewportBounds};

use crate::{
    event::{EventHandleError, EventHandleSuccess},
    EventContext, EventHandler, UserEvent,
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
    ) -> Result<EventHandleSuccess, EventHandleError>;
}

impl EventHandler for Viewport {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        ev: &crate::UserEvent,
    ) -> Result<crate::event::EventHandleSuccess, crate::event::EventHandleError> {
        use pp_editor::viewport::ViewportContent;
        match &mut self.content {
            ViewportContent::Folding(vp) => vp.handle_event(ctx, ev, &self.bounds),
            ViewportContent::Cutting(vp) => vp.handle_event(ctx, ev, &self.bounds),
        }
    }
}
