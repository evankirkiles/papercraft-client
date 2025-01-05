use pp_core::viewport::Viewport2D;
use winit::{dpi::PhysicalPosition, event::WindowEvent};

use crate::input::InputState;

use super::ViewportInput;

impl ViewportInput for Viewport2D {
    fn handle_event(
        &mut self,
        event: &WindowEvent,
        input_state: &InputState,
    ) -> Result<(), super::EventStoppedPropagation> {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                if input_state.mb3_pressed {
                    self.camera.pan(
                        position.x - input_state.cursor_pos.x,
                        position.y - input_state.cursor_pos.y,
                    );
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    // Standard scroll events should dolly in/out
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (*x as f64, *y as f64), // Touch "wheel" events should orbit
                    winit::event::MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                        (*x, *y)
                    }
                };
                self.camera.pan(dx, dy);
            }
            WindowEvent::PinchGesture { delta, .. } => {
                self.camera.dolly(delta * 50.0);
            }
            _ => (),
        };
        Ok(())
    }
}
