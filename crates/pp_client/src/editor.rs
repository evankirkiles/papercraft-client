use pp_core::{
    clear_cuts::ClearCutsCommand, make_cuts::MakeCutsCommand, select::SelectionActionType,
    select_elements::SelectCommand, update_flaps::UpdateFlapsCommand, CommandType,
};
use pp_editor::{
    tool::{SelectBoxTool, Tool},
    Editor,
};
use pp_save::save::Saveable;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, Url};

use crate::{
    event::{self, EventHandleSuccess},
    keyboard, EventHandler,
};

/// Triggers a file download in the browser
fn trigger_download(data: &[u8], filename: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Create a Blob from the data
    let array = js_sys::Uint8Array::from(data);
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array);
    let blob = Blob::new_with_u8_array_sequence(&blob_parts)?;

    // Create an object URL for the blob
    let url = Url::create_object_url_with_blob(&blob)?;

    // Create a temporary anchor element and click it
    let anchor = document.create_element("a")?.dyn_into::<web_sys::HtmlAnchorElement>()?;
    anchor.set_href(&url);
    anchor.set_download(filename);

    document.body().ok_or("No body")?.append_child(&anchor)?;
    anchor.click();
    document.body().ok_or("No body")?.remove_child(&anchor)?;

    // Clean up the object URL
    Url::revoke_object_url(&url)?;

    Ok(())
}

impl EventHandler for Editor {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        event: &crate::UserEvent,
    ) -> Option<Result<crate::event::EventHandleSuccess, crate::event::EventHandleError>> {
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
                // S: Mark edge as cut or Save (CMD+S)
                "KeyS" => {
                    if ctx.modifiers.super_pressed() {
                        // CMD+S: Save state as GLB file download
                        let _ = ctx
                            .state
                            .borrow()
                            .save()
                            .and_then(|save_file| save_file.to_binary())
                            .inspect_err(|e| log::error!("Failed to save state: {:?}", e))
                            .inspect(|glb_data| {
                                let _ = trigger_download(glb_data, "workspace.glb");
                            });
                        return Some(Ok(event::EventHandleSuccess::stop_propagation()));
                    } else {
                        // S: Mark edge as cut
                        let state = &mut ctx.state.borrow_mut();
                        ctx.history.borrow_mut().add(match ctx.modifiers.alt_pressed() {
                            false => CommandType::MakeCuts(MakeCutsCommand::from_select(state)),
                            true => CommandType::ClearCuts(ClearCutsCommand::from_select(state)),
                        })
                    };
                }
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
                return ctx.last_mouse_pos.map(|mouse_pos| {
                    Ok(EventHandleSuccess::set_tool(Some(Tool::SelectBox(SelectBoxTool {
                        start_pos: mouse_pos * ctx.surface_dpi,
                        end_pos: mouse_pos * ctx.surface_dpi,
                        action: match ctx.modifiers.shift_pressed() {
                            true => SelectionActionType::Invert,
                            false => SelectionActionType::Select,
                        },
                        is_dirty: true,
                    }))))
                });
            }
            _ => {}
        };
        None
    }
}
