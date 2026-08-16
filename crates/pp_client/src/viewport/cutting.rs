use pp_editor::tool::Tool;
use pp_editor::viewport::{cutting::CuttingViewport, ViewportBounds};

use crate::{
    event::{self, EventHandleError, EventHandleSuccess, UserEvent},
    keyboard,
};

use super::ViewportEventHandler;

impl ViewportEventHandler for CuttingViewport {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        _editor_state: &mut pp_editor::state::EditorState,
        ev: &crate::UserEvent,
        bounds: &ViewportBounds,
    ) -> Option<Result<crate::event::EventHandleSuccess, crate::event::EventHandleError>> {
        match ev {
            UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(
                keyboard::Key::Character(char),
            )) => match char.as_str() {
                // G: Translate
                "KeyG" => {
                    return Some(
                        self.create_tool_translate(&ctx.state.borrow(), bounds)
                            .map(|tool| EventHandleSuccess::set_tool(Some(Tool::Translate(tool))))
                            .map_err(|_| EventHandleError::default()),
                    );
                }
                // R: Rotate
                "KeyR" => {
                    return Some(
                        self.create_tool_rotate(&ctx.state.borrow(), bounds)
                            .map(|tool| EventHandleSuccess::set_tool(Some(Tool::Rotate(tool))))
                            .map_err(|_| EventHandleError::default()),
                    );
                }
                _ => {}
            },
            UserEvent::MouseWheel { delta } => {
                if ctx.modifiers.super_pressed() {
                    let fit_radius = ctx.state.borrow().world_bounds().bounding_radius();
                    self.camera.zoom(delta.y * 0.5, fit_radius);
                } else {
                    self.camera.pan(delta);
                };
            }
            _ => {}
        };
        None
    }
}
