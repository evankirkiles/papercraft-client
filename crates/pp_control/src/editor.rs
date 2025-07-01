use pp_core::{
    cut_edges::CutEdgesCommand, select::SelectionActionType, select_elements::SelectCommand,
    update_flaps::UpdateFlapsCommand,
};
use pp_editor::{
    tool::{SelectBoxTool, Tool},
    Editor,
};

use crate::{
    event::{self, EventHandleSuccess},
    keyboard, EventHandler,
};

impl EventHandler for Editor {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        event: &crate::UserEvent,
    ) -> Result<crate::event::EventHandleSuccess, crate::event::EventHandleError> {
        match event {
            // Select / cut keybinds
            event::UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(
                keyboard::Key::Character(char),
            )) => match char.as_str() {
                // A: Select all
                "KeyA" => ctx.history.borrow_mut().add(pp_core::CommandType::Select(
                    SelectCommand::select_all(
                        &mut ctx.state.borrow_mut(),
                        match ctx.modifiers.alt_pressed() {
                            true => pp_core::select::SelectionActionType::Deselect,
                            false => pp_core::select::SelectionActionType::Select,
                        },
                    ),
                )),
                // S: Mark edge as cut
                "KeyS" => ctx.history.borrow_mut().add(pp_core::CommandType::CutEdges(
                    CutEdgesCommand::cut_edges(
                        &mut ctx.state.borrow_mut(),
                        match ctx.modifiers.alt_pressed() {
                            true => pp_core::cut::CutActionType::Join,
                            false => pp_core::cut::CutActionType::Cut,
                        },
                    ),
                )),
                // D: Swap edge flap side
                "KeyD" => ctx.history.borrow_mut().add(pp_core::CommandType::UpdateFlaps(
                    UpdateFlapsCommand::swap_flaps(&mut ctx.state.borrow_mut()),
                )),
                // CMD+Z: Undo / redo
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
            // Left clicks "submit" selection queries through the GPU. Every
            // frame, the GPU is polled for the completion of such queries in
            // `draw` - when a query is ready, its action is parsed, all the
            // requested items are selected, and the buffer remains mapped
            // until any change in the viewport occurs.
            event::UserEvent::MouseInput(event::MouseInputEvent::Down(
                event::MouseButton::Left,
            )) => {
                return Ok(EventHandleSuccess::set_tool(Some(Tool::SelectBox(SelectBoxTool {
                    start_pos: ctx.last_mouse_pos.unwrap() * ctx.surface_dpi,
                    end_pos: ctx.last_mouse_pos.unwrap() * ctx.surface_dpi,
                    action: if ctx.modifiers.shift_pressed() {
                        SelectionActionType::Invert
                    } else {
                        SelectionActionType::Select
                    },
                }))));
            }
            _ => {}
        };
        Ok(Default::default())
    }
}
