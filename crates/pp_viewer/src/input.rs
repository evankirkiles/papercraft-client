use winit::dpi::PhysicalPosition;

/// Represents ephemeral user input states
#[derive(Default)]
pub struct InputState {
    pub mb1_pressed: bool,
    pub mb3_pressed: bool,
    pub shift_pressed: bool,
    pub cursor_pos: PhysicalPosition<f64>,
    pub is_touch: bool,
}
