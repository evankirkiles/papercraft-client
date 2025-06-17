use std::ops::DerefMut;

use crate::{
    event::{self, EventHandler, InternalEventHandleSuccess, PhysicalPosition},
    keyboard,
};
use pp_core::{
    commands::select::SelectCommand,
    id::{self, Id},
    select::SelectionActionType,
    settings::SelectionMode,
};
use pp_draw::select::{self, PixelData, SelectionMask, SelectionQueryArea, SelectionQueryResult};

#[derive(Debug, Default, Clone, Copy)]
pub struct SelectTool {}

impl SelectTool {
    /// Selects / unselects all elements in the mesh
    fn select_all(&self, ctx: &crate::event::EventContext, action: SelectionActionType) {
        let mut state = ctx.state.borrow_mut();
        let prev_state = state.selection.clone();
        state.select_all(action);
        ctx.history.borrow_mut().add(pp_core::CommandType::Select(SelectCommand {
            after: state.selection.clone(),
            before: prev_state,
        }))
    }

    /// Selects a single element vert / edge / face / piece under the mouse.
    fn select_single(
        &self,
        ctx: &crate::event::EventContext,
        cursor_pos: PhysicalPosition<f64>,
        action: SelectionActionType,
    ) -> Result<(), ()> {
        let query = {
            let state = ctx.state.borrow();
            let select_radius = match state.settings.selection_mode {
                SelectionMode::Face | SelectionMode::Piece => 2.0, // Face / piece selection is near-exact
                _ => 50.0,                                         // Vert / edge selection is fuzzy
            } * ctx.surface_dpi;
            select::SelectionQueryArea {
                rect: select::SelectionRect {
                    x: (cursor_pos.x * ctx.surface_dpi - select_radius).max(0.0) as u32,
                    y: (cursor_pos.y * ctx.surface_dpi - select_radius).max(0.0) as u32,
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
                let mut nearest: Option<(PixelData, f32)> = None;
                let center_x = (2 * area.rect.x + area.rect.width) as f32 / 2.0;
                let center_y = (2 * area.rect.y + area.rect.height) as f32 / 2.0;
                // TODO: Slice down to a smaller section which contains our pixels
                result.pixels.iter().for_each(|(x, y, pixel_data)| {
                    let distance = (x - center_x).powi(2) + (y - center_y).powi(2);
                    if let Some(nearest) = nearest {
                        if distance >= nearest.1 {
                            return;
                        }
                    }
                    nearest = Some((*pixel_data, distance));
                });
                let Some((pixel_data, _)) = nearest else { return };
                let mesh_id = id::MeshId::new(pixel_data.mesh_id - 1);
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
                        if pixel_data.p_id != 0 {
                            let piece_id = id::PieceId::new(pixel_data.p_id - 1);
                            state.select_piece(&(mesh_id, piece_id), action, true, true);
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
                        let action = match ctx.modifiers.alt_pressed() {
                            true => pp_core::select::SelectionActionType::Deselect,
                            false => pp_core::select::SelectionActionType::Select,
                        };
                        self.select_all(ctx, action);
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
                    "KeyD" => {
                        let mut state = ctx.state.borrow_mut();
                        let edges: Vec<_> = state.selection.edges.iter().copied().collect();
                        edges.iter().for_each(|id| state.swap_edge_flap(id));
                    }
                    "KeyZ" => {
                        if ctx.modifiers.super_pressed() {
                            let mut state = ctx.state.borrow_mut();
                            let mut history = ctx.history.borrow_mut();
                            if ctx.modifiers.shift_pressed() {
                                let _ = history.redo(&mut state);
                            } else {
                                let _ = history.undo(&mut state);
                            }
                        }
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
            event::UserEvent::MouseInput(event::MouseInputEvent::Up(button)) => {
                if let event::MouseButton::Left = button {
                    let cursor_pos = ctx.last_mouse_pos.unwrap();
                    let action = if ctx.modifiers.shift_pressed() {
                        SelectionActionType::Invert
                    } else {
                        SelectionActionType::Select
                    };
                    self.select_single(ctx, cursor_pos, action);
                }
                return Ok(InternalEventHandleSuccess::stop_internal_propagation());
            }
            _ => (),
        };
        Ok(InternalEventHandleSuccess::default())
    }
}
