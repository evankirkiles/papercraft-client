use std::ops::DerefMut;

use crate::{
    event::{self, EventHandler, InternalEventHandleSuccess},
    keyboard,
};
use pp_core::settings::SelectionMode;
use pp_draw::select;

#[derive(Debug, Default, Clone, Copy)]
pub struct SelectTool {}

impl EventHandler for SelectTool {
    fn handle_event(
        &mut self,
        ctx: &crate::event::EventContext,
        event: &crate::event::UserEvent,
    ) -> Result<event::InternalEventHandleSuccess, event::InternalEventHandleError> {
        match event {
            // Select / cut keybinds
            event::UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(key)) => match key {
                keyboard::Key::Character(char) => match char.as_str() {
                    "KeyA" => {
                        let mut state = ctx.state.borrow_mut();
                        state.select_all(match ctx.modifiers.alt_pressed() {
                            true => pp_core::select::SelectionActionType::Deselect,
                            false => pp_core::select::SelectionActionType::Select,
                        });
                        return Ok(InternalEventHandleSuccess::stop_propagation());
                    }
                    // TODO: Move this somewhere else, not "select" related
                    "KeyS" => {
                        let mut state = ctx.state.borrow_mut();
                        let edges: Vec<_> = state.selection.edges.iter().copied().collect();
                        state.cut_edges(
                            &edges[..],
                            match ctx.modifiers.alt_pressed() {
                                true => pp_core::cut::CutActionType::Join,
                                false => pp_core::cut::CutActionType::Cut,
                            },
                            pp_core::cut::CutMaskType::SelectionBorder,
                        );
                    }
                    _ => {}
                },
                _ => {}
            },
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
            event::UserEvent::MouseInput(event::MouseInputEvent::Down(button)) => {
                if let event::MouseButton::Left = button {
                    let state = ctx.state.borrow();
                    let mut renderer = ctx.renderer.borrow_mut();
                    let Some(renderer) = renderer.deref_mut() else {
                        return Ok(event::InternalEventHandleSuccess::stop_internal_propagation());
                    };
                    // Face / piece selection is exact, not fuzzy
                    let select_radius = match state.settings.selection_mode {
                        SelectionMode::Face | SelectionMode::Piece => 2.0,
                        _ => 50.0,
                    } * ctx.surface_dpi;
                    let cursor_pos = ctx.last_mouse_pos.unwrap();
                    renderer
                        .select_query(select::SelectionQuery {
                            action: Some(if ctx.modifiers.shift_pressed() {
                                select::SelectImmediateAction::NearestToggle
                            } else {
                                select::SelectImmediateAction::Nearest
                            }),
                            mask: match state.settings.selection_mode {
                                SelectionMode::Vert => pp_draw::select::SelectionMask::VERTS,
                                SelectionMode::Edge => pp_draw::select::SelectionMask::EDGES,
                                SelectionMode::Face => pp_draw::select::SelectionMask::FACES,
                                SelectionMode::Piece => pp_draw::select::SelectionMask::PIECES,
                            },
                            rect: select::SelectionRect {
                                x: (cursor_pos.x * ctx.surface_dpi - select_radius).max(0.0) as u32,
                                y: (cursor_pos.y * ctx.surface_dpi - select_radius).max(0.0) as u32,
                                width: select_radius as u32 * 2,
                                height: select_radius as u32 * 2,
                            },
                        })
                        .unwrap();
                }
                return Ok(InternalEventHandleSuccess::stop_internal_propagation());
            }
            _ => (),
        };
        Ok(InternalEventHandleSuccess::default())
    }
}
