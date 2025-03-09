# pp_control

<!--toc:start-->

- [pp-control](#pp-control)
<!--toc:end-->

The controller mediates communication between the user, model, and the view
layer. It directly listens for high-frequency commands (like mouse events
executed on the canvas and redraw requests) with `winit`.

The `winit` event loop is further enhanced with the ability to take in user
`commands` from the UI - actions which may modify state. If state is modified,
`pp_control` sends back `events` to registered callbacks which allow external
UI layers to react to state changes internal to `pp_core`.
