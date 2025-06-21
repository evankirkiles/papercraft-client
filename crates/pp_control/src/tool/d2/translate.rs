use pp_core::{tool::ToolContext, transform_pieces::TransformPiecesCommand};

use crate::{
    event::{self, EventHandler, MouseButton, PhysicalPosition, PointerEvent},
    keyboard,
};

#[derive(Debug, Clone, Copy)]
pub struct TranslateTool {
    last_pos: Option<PhysicalPosition<f64>>,
    tool: pp_core::tool::TransformTool,
}

impl TranslateTool {
    pub fn new(ctx: ToolContext) -> Self {
        Self { last_pos: None, tool: pp_core::tool::TransformTool::new(ctx) }
    }
}

impl EventHandler for TranslateTool {
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
            // LMB click "accepts" the changes, removing the translate tool and
            // adding an entry onto the history stack for undoing the changes
            event::UserEvent::MouseInput(event::MouseInputEvent::Up(button)) => match button {
                MouseButton::Left => {
                    let pieces: Vec<_> =
                        ctx.state.borrow().selection.pieces.iter().copied().collect();
                    ctx.history.borrow_mut().add(pp_core::CommandType::TransformPieces(
                        TransformPiecesCommand { pieces, delta: self.tool.transform },
                    ));
                    return Ok(event::InternalEventHandleSuccess::clear_tool());
                }
                // RMB click cancels the tool
                MouseButton::Right => {
                    self.tool.reset(&mut ctx.state.borrow_mut());
                    return Ok(event::InternalEventHandleSuccess::clear_tool());
                }
                _ => (),
            },
            // On ESC, clear the tool entirely as if it didn't happen
            event::UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(
                keyboard::Key::Named(keyboard::NamedKey::Escape),
            )) => {
                self.tool.reset(&mut ctx.state.borrow_mut());
                return Ok(event::InternalEventHandleSuccess::clear_tool());
            }
            _ => (),
        };
        Ok(event::InternalEventHandleSuccess::stop_propagation())
    }
}
