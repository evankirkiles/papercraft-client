use pp_core::mesh::MeshElementType;
use pp_editor::viewport::{folding::FoldingViewport, ViewportBounds};

use crate::event;

use super::ViewportEventHandler;

impl ViewportEventHandler for FoldingViewport {
    fn handle_event(
        &mut self,
        ctx: &crate::EventContext,
        ev: &crate::UserEvent,
        _: &ViewportBounds,
    ) -> Result<crate::event::EventHandleSuccess, crate::event::EventHandleError> {
        use event::UserEvent;
        match ev {
            // UserEvent::KeyboardInput(event::KeyboardInputEvent::Down(key)) => match key {
            //     keyboard::Key::Named(keyboard::NamedKey::Tab) => {
            //         let mut state = ctx.state.borrow_mut();
            //         state.viewport_3d.xray_mode = !state.viewport_3d.xray_mode;
            //     }
            //     _ => (),
            // },
            UserEvent::MouseWheel { delta } => {
                let mut state = ctx.state.borrow_mut();
                if ctx.modifiers.shift_pressed() {
                    self.camera.pan(delta);
                } else if ctx.modifiers.super_pressed() {
                    self.camera.dolly(delta.y * 0.5);
                } else if ctx.modifiers.ctrl_pressed() {
                    let new_t = (state.settings.t + (delta.y * 0.01)).clamp(0.0, 1.0);
                    state.settings.t = new_t;
                    state.meshes.iter_mut().for_each(|(_, mesh)| {
                        mesh.elem_dirty |= MeshElementType::all();
                        mesh.pieces.iter_mut().for_each(|(_, piece)| {
                            piece.t = new_t;
                            piece.elem_dirty = true;
                        });
                    });
                    return Ok(event::EventHandleSuccess::stop_propagation());
                } else {
                    self.camera.orbit(delta);
                }
            }
            _ => {}
        };
        Ok(Default::default())
    }
}
