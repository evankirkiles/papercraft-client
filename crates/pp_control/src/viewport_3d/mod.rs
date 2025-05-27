use crate::event::EventHandler;
use pp_core::mesh::MeshElementType;

use crate::event;
use crate::keyboard;
use crate::tool;

#[derive(Debug, Clone, Copy)]
pub enum Controller3DTool {
    Select(tool::SelectTool),
}

impl Default for Controller3DTool {
    fn default() -> Self {
        Self::Select(tool::SelectTool::default())
    }
}

/// The Controller3D modifies state based on input to the 3D viewport pane.
#[derive(Debug, Default, Clone, Copy)]
pub struct Controller3D {
    // The active tool.
    tool: Controller3DTool,
}

impl Controller3D {
    fn handle_event_with_tool(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::InternalEventHandleSuccess, event::InternalEventHandleError> {
        let res = match self.tool {
            Controller3DTool::Select(ref mut tool) => tool.handle_event(ctx, ev),
        };
        if res.is_ok_and(|e| e.clear_tool) {
            self.tool = Default::default()
        }
        res
    }
}

impl event::EventHandler for Controller3D {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::InternalEventHandleSuccess, event::InternalEventHandleError> {
        // First, call any active tool's event handler, potentially ending propagation
        let res = self.handle_event_with_tool(ctx, ev)?;
        if res.stop_propagation {
            return Ok(res);
        }

        // If no tool took the event, pass it to the viewport handler itself,
        // as it can influence the camera still.
        match ev {
            event::UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(key)) => match key {
                keyboard::Key::Named(keyboard::NamedKey::Tab) => {
                    let mut state = ctx.state.borrow_mut();
                    state.viewport_3d.xray_mode = !state.viewport_3d.xray_mode;
                }
                _ => (),
            },
            event::UserEvent::MouseWheel { dx, dy } => {
                let mut state = ctx.state.borrow_mut();
                if ctx.modifiers.shift_pressed() {
                    state.viewport_3d.camera.pan(*dx, *dy);
                } else if ctx.modifiers.super_pressed() {
                    state.viewport_3d.camera.dolly(*dy * 0.5);
                } else if ctx.modifiers.ctrl_pressed() {
                    let new_t = (state.settings.t + ((*dy) as f32 * 0.01)).clamp(0.0, 1.0);
                    state.settings.t = new_t;
                    state.meshes.iter_mut().for_each(|(_, mesh)| {
                        mesh.elem_dirty |= MeshElementType::all();
                        mesh.pieces.iter_mut().for_each(|(_, piece)| {
                            piece.t = new_t;
                            piece.elem_dirty = true;
                        });
                    });
                    return Ok(event::InternalEventHandleSuccess::stop_propagation());
                } else {
                    state.viewport_3d.camera.orbit(*dx, *dy);
                }
            }
            _ => {}
        };
        Ok(event::InternalEventHandleSuccess::stop_propagation())
    }
}
