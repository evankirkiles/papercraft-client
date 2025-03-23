use std::ops::DerefMut;

use crate::event::{self, EventHandleSuccess, EventHandler};
use pp_draw::select;

const SELECT_RADIUS: f64 = 50.0;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SelectTool {}

impl EventHandler for SelectTool {
    fn handle_event(
        &mut self,
        ctx: &crate::event::EventContext,
        event: &crate::event::UserEvent,
    ) -> Result<crate::event::EventHandleSuccess, crate::event::EventHandleError> {
        match event {
            // Left clicks "submit" selection queries through the GPU. Every
            // frame, the GPU is polled for the completion of such queries in
            // `draw` - when a query is ready, its action is parsed, all the
            // requested items are selected, and the buffer remains mapped
            // until any change in the viewport occurs.
            //
            // Note that "preparing" a query asynchronously is different from
            // actually "USING" a query. This is because sometimes we don't want
            // to select immediately and would rather select sub-sections of a frozen
            // view (e.g. in selection painting).
            event::UserEvent::MouseInput(event::MouseInputEvent::Up(button)) => {
                let event::MouseButton::Left = button else { return Ok(Default::default()) };
                let mut renderer = ctx.renderer.borrow_mut();
                let Some(renderer) = renderer.deref_mut() else {
                    return Ok(EventHandleSuccess::StopPropagation);
                };
                let select_radius = SELECT_RADIUS * ctx.surface_dpi;
                let cursor_pos = ctx.last_mouse_pos.unwrap();
                #[allow(unused_must_use)]
                renderer.select_query(select::SelectionQuery {
                    action: Some(if ctx.modifiers.shift_pressed() {
                        select::SelectImmediateAction::NearestToggle
                    } else {
                        select::SelectImmediateAction::Nearest
                    }),
                    mask: pp_draw::select::SelectionMask::all(),
                    rect: select::SelectionRect {
                        x: (cursor_pos.x * ctx.surface_dpi - select_radius).max(0.0) as u32,
                        y: (cursor_pos.y * ctx.surface_dpi - select_radius).max(0.0) as u32,
                        width: select_radius as u32 * 2,
                        height: select_radius as u32 * 2,
                    },
                });
                Ok(EventHandleSuccess::StopPropagation)
            }
            _ => Ok(Default::default()),
        }
    }
}
