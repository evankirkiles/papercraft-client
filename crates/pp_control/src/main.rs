fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = winit::event_loop::EventLoop::with_user_event().build()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = pp_control::App::new(event_loop.create_proxy());
    event_loop.run_app(&mut app)?;
    Ok(())
}
