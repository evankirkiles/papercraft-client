use std::ops::DerefMut;

use cgmath::{MetricSpace, Point2};
use pp_core::measures::Rect;
use pp_core::{select::SelectionActionType, select_elements::SelectCommand, MeshId};
use pp_draw::select::{self, PixelData, SelectionMask, SelectionQueryArea, SelectionQueryResult};
use pp_editor::state::SelectionMode;
use slotmap::KeyData;

use crate::{
    event::{self, EventHandler, MouseButton, PointerEvent},
    keyboard,
    tool::{apply_pixel, selection_mask},
};

impl EventHandler for pp_editor::tool::SelectBoxTool {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        editor_state: &mut pp_editor::state::EditorState,
        event: &crate::UserEvent,
    ) -> Option<Result<event::EventHandleSuccess, event::EventHandleError>> {
        match event {
            // On mouse move, update the end pos
            event::UserEvent::Pointer(PointerEvent::Move(pos)) => {
                self.update(*pos * ctx.surface_dpi);
                return Some(Ok(event::EventHandleSuccess::stop_internal_propagation()));
            }
            event::UserEvent::MouseInput(event::MouseInputEvent::Up(button)) => match button {
                // LMB "accepts" the tool changes, removing the translate tool and
                // adding an entry onto the history stack for undoing the changes
                MouseButton::Left => {
                    if self.start_pos.distance(self.end_pos) < 10.0 * ctx.surface_dpi {
                        let _ = self.select_single(ctx, editor_state);
                    } else {
                        let _ = self.select_multiple(ctx, editor_state);
                    }
                    return Some(Ok(event::EventHandleSuccess::set_tool(None)));
                }
                // RMB: Cancel
                MouseButton::Right => {
                    return Some(Ok(event::EventHandleSuccess::set_tool(None)));
                }
                _ => {}
            },
            // ESC: Cancel
            event::UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(
                keyboard::Key::Named(keyboard::NamedKey::Escape),
            )) => {
                return Some(Ok(event::EventHandleSuccess::set_tool(None)));
            }
            _ => {}
        };
        Some(Ok(event::EventHandleSuccess::stop_internal_propagation()))
    }
}

pub trait MultiselectTool {
    fn get_cursor_pos(&self) -> cgmath::Point2<f32>;
    fn get_action(&self) -> SelectionActionType;
    fn select_multiple(
        &mut self,
        ctx: &event::EventContext,
        editor_state: &pp_editor::state::EditorState,
    ) -> Result<(), ()>;

    fn select_single(
        &self,
        ctx: &event::EventContext,
        editor_state: &pp_editor::state::EditorState,
    ) -> Result<(), ()> {
        let action = self.get_action();
        let cursor_pos = self.get_cursor_pos();
        let query = {
            let select_radius = match editor_state.selection_mode {
                SelectionMode::Face | SelectionMode::Piece => 2.0, // Face / piece selection is near-exact
                _ => 50.0,                                         // Vert / edge selection is fuzzy
            } * ctx.surface_dpi;
            select::SelectionQueryArea {
                rect: Rect {
                    x: (cursor_pos.x - select_radius).max(0.0) as u32,
                    y: (cursor_pos.y - select_radius).max(0.0) as u32,
                    width: select_radius as u32 * 2,
                    height: select_radius as u32 * 2,
                },
                mask: selection_mask(&editor_state.selection_mode),
            }
        };
        let callback = {
            let state = ctx.state.clone();
            let history = ctx.history.clone();
            move |area: &SelectionQueryArea, result: &SelectionQueryResult| {
                let mut state = state.borrow_mut();
                let prev_state = state.selection.clone();
                // Actions which are not "Invert" clear the selection state
                // NOTE: This might fit better in a different place
                if action != SelectionActionType::Invert {
                    state.select_all(SelectionActionType::Deselect);
                }
                // TODO: Slice down to a smaller section which contains our pixels
                let mut nearest: Option<(PixelData, f32)> = None;
                let center = Point2 {
                    x: (area.rect.x + area.rect.width / 2) as f32,
                    y: (area.rect.y + area.rect.height / 2) as f32,
                };
                result.pixels.iter().for_each(|(pos, pixel_data)| {
                    let distance = center.distance(*pos);
                    if let Some(nearest) = nearest {
                        if distance >= nearest.1 {
                            return;
                        }
                    }
                    nearest = Some((*pixel_data, distance));
                });
                let Some((pixel_data, _)) = nearest else { return };
                apply_pixel(&mut state, result.area.mask, &pixel_data, action, true);
                // Add the selection command onto the undo/redo stack
                history.borrow_mut().add(pp_core::CommandType::Select(SelectCommand {
                    after: Box::new(state.selection.clone()),
                    before: Box::new(prev_state),
                }))
            }
        };
        let mut renderer = ctx.renderer.borrow_mut();
        let Some(renderer) = renderer.deref_mut() else {
            return Err(());
        };
        renderer.select_query(query, Box::new(callback)).map_err(|_| ())
    }
}

impl MultiselectTool for pp_editor::tool::SelectBoxTool {
    fn select_multiple(
        &mut self,
        ctx: &event::EventContext,
        editor_state: &pp_editor::state::EditorState,
    ) -> Result<(), ()> {
        let query = select::SelectionQueryArea {
            rect: Rect::between(self.start_pos, self.end_pos).into(),
            mask: selection_mask(&editor_state.selection_mode),
        };
        let callback = {
            let state = ctx.state.clone();
            let history = ctx.history.clone();
            let action = self.action;
            move |area: &SelectionQueryArea, result: &SelectionQueryResult| {
                let mut state = state.borrow_mut();
                let prev_state = state.selection.clone();
                // Actions which are not "Invert" clear the selection state
                // NOTE: This might fit better in a different place
                if action != SelectionActionType::Invert {
                    state.select_all(SelectionActionType::Deselect);
                }
                // Collect all the pixels found in the box
                let rect: Rect<f32> = area.rect.into();
                let mut elements: Vec<_> = result
                    .pixels
                    .iter()
                    .filter(|(pos, _)| rect.contains(pos))
                    .map(|(_, pixel_data)| pixel_data)
                    .collect();
                elements.dedup_by_key(|pixel| {
                    let mesh_id: MeshId = KeyData::from_ffi(pixel.mesh_id).into();
                    match result.area.mask {
                        SelectionMask::VERTS | SelectionMask::EDGES => (mesh_id, pixel.el_id),
                        SelectionMask::FACES | SelectionMask::PIECES => (mesh_id, pixel.f_id),
                        _ => (mesh_id, pixel.f_id),
                    }
                });
                // Now select all of them
                elements.iter().for_each(|pixel| {
                    apply_pixel(&mut state, result.area.mask, pixel, action, false);
                });
                // Add the selection command onto the undo/redo stack
                history.borrow_mut().add(pp_core::CommandType::Select(SelectCommand {
                    after: Box::new(state.selection.clone()),
                    before: Box::new(prev_state),
                }))
            }
        };
        let mut renderer = ctx.renderer.borrow_mut();
        let Some(renderer) = renderer.deref_mut() else {
            return Err(());
        };
        renderer.select_query(query, Box::new(callback)).map_err(|_| ())
    }

    fn get_cursor_pos(&self) -> cgmath::Point2<f32> {
        self.end_pos
    }

    fn get_action(&self) -> SelectionActionType {
        self.action
    }
}
