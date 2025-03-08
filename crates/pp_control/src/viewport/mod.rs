use crate::input::InputState;

pub mod d2;
pub mod d3;

pub struct EventStoppedPropagation;

pub trait ViewportInput {
    fn handle_event(
        &mut self,
        event: &winit::event::WindowEvent,
        input_state: &InputState,
    ) -> Result<(), EventStoppedPropagation>;
}
