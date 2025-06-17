use pp_core::{tool::ToolContext, transform_piece::TransformPiecesCommand};

use crate::{
    event::{self, EventHandler, MouseButton, PhysicalPosition, PointerEvent},
    keyboard,
};

#[derive(Debug, Clone, Copy)]
pub struct TransformTool {
    last_pos: Option<PhysicalPosition<f64>>,
    tool: pp_core::tool::TransformTool,
}

impl TransformTool {
    pub fn new(ctx: ToolContext) -> Self {
        Self { last_pos: None, tool: pp_core::tool::TransformTool::new(ctx) }
    }
}

impl EventHandler for TransformTool {
    fn handle_event(
        &mut self,
        ctx: &event::EventContext,
        event: &event::UserEvent,
    ) -> Result<event::InternalEventHandleSuccess, event::InternalEventHandleError> {
        match event {
            // On mouse move, move the piece accordingly
            event::UserEvent::Pointer(e) => match e {
                PointerEvent::Exit => self.last_pos = None,
                PointerEvent::Move(pos) => {
                    if let Some(last_pos) = self.last_pos {
                        let PhysicalPosition { x, y } = pos;
                        let dx = x - last_pos.x;
                        let dy = y - last_pos.y;
                        self.tool.translate(&mut ctx.state.borrow_mut(), dx as f32, dy as f32);
                    }
                    self.last_pos = Some(*pos);
                }
                _ => (),
            },
            event::UserEvent::MouseInput(e) => match e {
                event::MouseInputEvent::Up(button) => match button {
                    // LMB click "accepts" the changes, removing the transform tool
                    MouseButton::Left => {
                        ctx.history.borrow_mut().add(pp_core::CommandType::TransformPieces(
                            TransformPiecesCommand {
                                pieces: ctx
                                    .state
                                    .borrow()
                                    .selection
                                    .pieces
                                    .iter()
                                    .copied()
                                    .collect(),
                                delta: self.tool.transform,
                            },
                        ));
                        return Ok(event::InternalEventHandleSuccess::clear_tool());
                    }
                    // RMB click resets any transform that occurred
                    MouseButton::Right => {
                        self.tool.reset(&mut ctx.state.borrow_mut());
                        return Ok(event::InternalEventHandleSuccess::clear_tool());
                    }
                    _ => (),
                },
                _ => (),
            },
            event::UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(key)) => match key {
                keyboard::Key::Named(char) => match char {
                    keyboard::NamedKey::Escape => {
                        self.tool.reset(&mut ctx.state.borrow_mut());
                        return Ok(event::InternalEventHandleSuccess::clear_tool());
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        };
        Ok(event::InternalEventHandleSuccess::stop_propagation())
    }
}
