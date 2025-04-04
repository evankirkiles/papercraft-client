use pp_core::id::Id;

use crate::event;
use crate::keyboard;

mod tool_select;

/// The Controller3D modifies state based on input to the 3D viewport pane.
#[derive(Debug, Default, Clone, Copy)]
pub struct Controller3D {
    // The active tool.
    tool: Controller3DTool,
}

impl event::EventHandler for Controller3D {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        // First, call any active tool's event handler, potentially ending propagation
        if self.tool.handle_event(ctx, ev)? == event::EventHandleSuccess::StopPropagation {
            return Ok(event::EventHandleSuccess::StopPropagation);
        }

        // If no tool took the event, pass it to the viewport handler itself,
        // as it can influence the camera still.
        match ev {
            event::UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(key)) => match key {
                keyboard::Key::Named(keyboard::NamedKey::Tab) => {
                    let mut state = ctx.state.borrow_mut();
                    state.viewport_3d.xray_mode = !state.viewport_3d.xray_mode;
                }
                keyboard::Key::Character(char) => match char.as_str() {
                    "KeyA" => {
                        let mut state = ctx.state.borrow_mut();
                        state.select_all(match ctx.modifiers.alt_pressed() {
                            true => pp_core::select::SelectionActionType::Deselect,
                            false => pp_core::select::SelectionActionType::Select,
                        });
                    }
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
                    _ => (),
                },
                _ => (),
            },
            event::UserEvent::MouseWheel { dx, dy } => {
                let mut state = ctx.state.borrow_mut();
                if ctx.modifiers.shift_pressed() {
                    state.viewport_3d.camera.pan(*dx, *dy);
                } else if ctx.modifiers.super_pressed() {
                    state.viewport_3d.camera.dolly(*dy * 0.5);
                } else {
                    state.viewport_3d.camera.orbit(*dx, *dy);
                }
            }
            _ => {}
        };
        Ok(event::EventHandleSuccess::StopPropagation)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Controller3DTool {
    Select(tool_select::SelectTool),
}

impl event::EventHandler for Controller3DTool {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        ev: &event::UserEvent,
    ) -> Result<event::EventHandleSuccess, event::EventHandleError> {
        match *self {
            Controller3DTool::Select(ref mut tool) => tool.handle_event(ctx, ev),
        }
    }
}

impl Default for Controller3DTool {
    fn default() -> Self {
        Self::Select(tool_select::SelectTool::default())
    }
}
