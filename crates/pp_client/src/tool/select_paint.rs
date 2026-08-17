use std::ops::DerefMut;

use pp_core::measures::Rect;
use pp_core::{select::SelectionActionType, select_elements::SelectCommand};
use pp_draw::select::{self, SelectionQueryArea, SelectionQueryResult};
use pp_editor::state::SelectTool;
use pp_editor::tool::SelectPaintTool;

use crate::{
    event::{self, EventHandler, MouseButton, PointerEvent},
    keyboard,
    tool::{apply_pixel, select_box::MultiselectTool, selection_mask},
};

/// How many surface pixels the brush grows per unit of wheel delta
const RADIUS_SPEED: f32 = 0.5;

/// The whole surface, which is the region the paint tool captures and then
/// re-uses for every hit test until something invalidates it.
fn capture_area(
    ctx: &event::EventContext,
    editor_state: &pp_editor::state::EditorState,
) -> SelectionQueryArea {
    select::SelectionQueryArea {
        rect: Rect {
            x: 0,
            y: 0,
            width: (ctx.surface_size.width * ctx.surface_dpi) as u32,
            height: (ctx.surface_size.height * ctx.surface_dpi) as u32,
        },
        mask: selection_mask(&editor_state.selection_mode),
    }
}

/// Warms the select buffer for the whole surface so the first stroke doesn't
/// have to wait on a GPU round trip. Called on entering paint mode, and again
/// whenever the selection mode changes out from under the cached buffer.
///
/// The buffer is invalidated first rather than trusted. `Renderer::prepare`
/// already drops it whenever the camera or geometry changes, but it only sees
/// those dirty flags once a frame - if the user orbits and enters paint mode
/// within the same frame, the cache would still be holding the old view.
pub(crate) fn capture_select_buffer(
    ctx: &event::EventContext,
    editor_state: &pp_editor::state::EditorState,
) {
    let mut renderer = ctx.renderer.borrow_mut();
    let Some(renderer) = renderer.deref_mut() else { return };
    renderer.select_invalidate();
    let _ = renderer.select_query(capture_area(ctx, editor_state), Box::new(|_: &_, _: &_| {}));
}

impl EventHandler for SelectPaintTool {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        editor_state: &mut pp_editor::state::EditorState,
        event: &crate::UserEvent,
    ) -> Option<Result<event::EventHandleSuccess, event::EventHandleError>> {
        match event {
            // Follow the cursor, painting continuously if a stroke is in progress
            event::UserEvent::Pointer(PointerEvent::Move(pos)) => {
                self.update(*pos * ctx.surface_dpi);
                if self.action.is_some() {
                    let _ = self.select_multiple(ctx, editor_state);
                }
            }
            event::UserEvent::MouseInput(event::MouseInputEvent::Down(MouseButton::Left)) => {
                // Snapshot the selection so the whole stroke lands on the
                // history stack as a single undo step
                self.stroke_start = Some(Box::new(ctx.state.borrow().selection.clone()));
                self.action = Some(match ctx.modifiers.shift_pressed() {
                    true => SelectionActionType::Deselect,
                    false => SelectionActionType::Select,
                });
                let _ = self.select_multiple(ctx, editor_state);
            }
            event::UserEvent::MouseInput(event::MouseInputEvent::Up(MouseButton::Left)) => {
                self.action = None;
                if let Some(before) = self.stroke_start.take() {
                    let after = Box::new(ctx.state.borrow().selection.clone());
                    ctx.history
                        .borrow_mut()
                        .add(pp_core::CommandType::Select(SelectCommand { before, after }));
                }
            }
            // The wheel resizes the brush rather than moving the camera, which
            // would invalidate the select buffer we captured on entry
            event::UserEvent::MouseWheel { delta } => {
                self.set_radius(self.radius + delta.y * RADIUS_SPEED, ctx.surface_dpi);
                return Some(Ok(event::EventHandleSuccess::stop_propagation()));
            }
            // ESC: Leave paint mode
            event::UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(
                keyboard::Key::Named(keyboard::NamedKey::Escape),
            )) => {
                editor_state.select_tool = SelectTool::Box;
                return Some(Ok(event::EventHandleSuccess::set_tool(None).mark_dirty()));
            }
            _ => {}
        };
        // Paint mode is modal: nothing else gets a say, so the select buffer we
        // captured on entry stays valid for the whole session.
        Some(Ok(event::EventHandleSuccess::stop_internal_propagation()))
    }
}

impl MultiselectTool for SelectPaintTool {
    /// Selects everything the brush circle currently covers.
    ///
    /// Unlike box select, the query covers the *entire* surface rather than the
    /// brush's bounding box. The first call renders and maps it; every call
    /// after that is served straight from the cached buffer by
    /// `SelectManager::query`, so a stroke costs no further GPU work.
    fn select_multiple(
        &mut self,
        ctx: &event::EventContext,
        editor_state: &pp_editor::state::EditorState,
    ) -> Result<(), ()> {
        let Some(action) = self.action else { return Ok(()) };
        let query = capture_area(ctx, editor_state);
        let callback = {
            let state = ctx.state.clone();
            // Captured by value: on the first (cold) call the buffer isn't
            // mapped yet, so this may not run until a frame or two later.
            let center = self.cursor_pos;
            let radius = self.radius;
            move |_: &SelectionQueryArea, result: &SelectionQueryResult| {
                let mut state = state.borrow_mut();
                // Note there's deliberately no `select_all(Deselect)` here, and
                // no history entry - a stroke accumulates, and is committed as
                // one command when the mouse comes back up.
                result.pixels_in_circle(center, radius).for_each(|pixel| {
                    apply_pixel(&mut state, result.area.mask, pixel, action, false);
                });
            }
        };
        let mut renderer = ctx.renderer.borrow_mut();
        let Some(renderer) = renderer.deref_mut() else {
            return Err(());
        };
        renderer.select_query(query, Box::new(callback)).map_err(|_| ())
    }

    fn get_cursor_pos(&self) -> cgmath::Point2<f32> {
        self.cursor_pos
    }

    fn get_action(&self) -> SelectionActionType {
        self.action.unwrap_or(SelectionActionType::Select)
    }
}
