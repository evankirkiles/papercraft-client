use std::ops::DerefMut;

use cgmath::{MetricSpace, Point2};
use pp_core::{
    id::{self, Id},
    select::SelectionActionType,
    select_elements::SelectCommand,
    settings::SelectionMode,
    MeshId,
};
use pp_draw::select::{self, PixelData, SelectionMask, SelectionQueryArea, SelectionQueryResult};
use pp_editor::measures::Rect;
use slotmap::KeyData;

use crate::{
    event::{self, EventHandler, MouseButton, PointerEvent},
    keyboard,
};

impl EventHandler for pp_editor::tool::SelectBoxTool {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        event: &crate::UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        match event {
            // On mouse move, update the end pos
            event::UserEvent::Pointer(PointerEvent::Move(pos)) => {
                self.end_pos = *pos * ctx.surface_dpi;
                return Ok(event::EventHandleSuccess::stop_internal_propagation());
            }
            event::UserEvent::MouseInput(event::MouseInputEvent::Up(button)) => match button {
                // LMB "accepts" the tool changes, removing the translate tool and
                // adding an entry onto the history stack for undoing the changes
                MouseButton::Left => {
                    if self.start_pos.distance(self.end_pos) < 10.0 * ctx.surface_dpi {
                        let _ = self.select_single(ctx);
                    } else {
                        let _ = self.select_multiple(ctx);
                    }
                    return Ok(event::EventHandleSuccess::set_tool(None));
                }
                // RMB: Cancel
                MouseButton::Right => {
                    return Ok(event::EventHandleSuccess::set_tool(None));
                }
                _ => {}
            },
            // ESC: Cancel
            event::UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(
                keyboard::Key::Named(keyboard::NamedKey::Escape),
            )) => {
                return Ok(event::EventHandleSuccess::set_tool(None));
            }
            _ => {}
        };
        Ok(event::EventHandleSuccess::stop_internal_propagation())
    }
}

pub trait MultiselectTool {
    fn get_cursor_pos(&self) -> cgmath::Point2<f32>;
    fn get_action(&self) -> SelectionActionType;
    fn select_multiple(&mut self, ctx: &event::EventContext) -> Result<(), ()>;

    fn select_single(&self, ctx: &event::EventContext) -> Result<(), ()> {
        let action = self.get_action();
        let cursor_pos = self.get_cursor_pos();
        let query = {
            let state = ctx.state.borrow();
            let select_radius = match state.settings.selection_mode {
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
                mask: match state.settings.selection_mode {
                    SelectionMode::Vert => pp_draw::select::SelectionMask::VERTS,
                    SelectionMode::Edge => pp_draw::select::SelectionMask::EDGES,
                    SelectionMode::Face => pp_draw::select::SelectionMask::FACES,
                    SelectionMode::Piece => pp_draw::select::SelectionMask::PIECES,
                },
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
                let mesh_id: MeshId = KeyData::from_ffi(pixel_data.mesh_id).into();
                match result.area.mask {
                    SelectionMask::VERTS => {
                        let vert_id = id::VertexId::new(pixel_data.el_id);
                        state.select_vert(&(mesh_id, vert_id), action, true);
                    }
                    SelectionMask::EDGES => {
                        let edge_id = id::EdgeId::new(pixel_data.el_id);
                        state.select_edge(&(mesh_id, edge_id), action, true, true);
                    }
                    SelectionMask::FACES => {
                        let face_id = id::FaceId::new(pixel_data.f_id);
                        state.select_face(&(mesh_id, face_id), action, true, true);
                    }
                    SelectionMask::PIECES => {
                        let face_id = id::FaceId::new(pixel_data.f_id);
                        let p_id = state.meshes[mesh_id].faces[face_id.to_usize()].p;
                        if let Some(p_id) = p_id {
                            state.select_piece(&(mesh_id, p_id), action);
                        }
                    }
                    _ => {}
                }
                // Add the selection command onto the undo/redo stack
                history.borrow_mut().add(pp_core::CommandType::Select(SelectCommand {
                    after: state.selection.clone(),
                    before: prev_state,
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
    fn select_multiple(&mut self, ctx: &event::EventContext) -> Result<(), ()> {
        let query = select::SelectionQueryArea {
            rect: Rect::between(self.start_pos, self.end_pos).into(),
            mask: match ctx.state.borrow().settings.selection_mode {
                SelectionMode::Vert => pp_draw::select::SelectionMask::VERTS,
                SelectionMode::Edge => pp_draw::select::SelectionMask::EDGES,
                SelectionMode::Face => pp_draw::select::SelectionMask::FACES,
                SelectionMode::Piece => pp_draw::select::SelectionMask::PIECES,
            },
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
                    let mesh_id: MeshId = KeyData::from_ffi(pixel.mesh_id).into();
                    match result.area.mask {
                        SelectionMask::VERTS => {
                            let vert_id = id::VertexId::new(pixel.el_id);
                            state.select_vert(&(mesh_id, vert_id), action, false);
                        }
                        SelectionMask::EDGES => {
                            let edge_id = id::EdgeId::new(pixel.el_id);
                            state.select_edge(&(mesh_id, edge_id), action, false, true);
                        }
                        SelectionMask::FACES => {
                            let face_id = id::FaceId::new(pixel.f_id);
                            state.select_face(&(mesh_id, face_id), action, false, true);
                        }
                        SelectionMask::PIECES => {
                            let face_id = id::FaceId::new(pixel.f_id);
                            let p_id = state.meshes[mesh_id].faces[face_id.to_usize()].p;
                            if let Some(p_id) = p_id {
                                state.select_piece(&(mesh_id, p_id), action);
                            }
                        }
                        _ => {}
                    }
                });
                // Add the selection command onto the undo/redo stack
                history.borrow_mut().add(pp_core::CommandType::Select(SelectCommand {
                    after: state.selection.clone(),
                    before: prev_state,
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
