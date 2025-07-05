import { App, Editor, PressedState } from "@paper/core";
import { ModifierKeys } from "./modifiers";

// TODO: Clean this up
const DOCUMENT_EVENTS = [
  "keydown",
  "keyup",
] as const satisfies (keyof HTMLElementEventMap)[];

const CANVAS_EVENTS = [
  "wheel",
  "pointermove",
  "mousedown",
  "mouseup",
] as const satisfies (keyof HTMLElementEventMap)[];

export default class PaperApp
  extends App
  implements
    Pick<
      HTMLCanvasElement,
      `on${(typeof CANVAS_EVENTS | typeof DOCUMENT_EVENTS)[number]}`
    >
{
  private _canvas: HTMLCanvasElement | undefined;
  private abortController: AbortController | undefined;
  private modifiers = new ModifierKeys();

  // The
  public editor: Editor = super.get_editor_snapshot();

  /**
   * Attachs the app to an HTML Canvas.
   *
   * On the Rust side, this retrieves the canvas's GPU context and allocates
   * GPU resources. On the JS side, this attaches listeners to the canvas and
   * to the window for controlling the Rust app.
   */
  async attach(canvas: HTMLCanvasElement) {
    if (this.abortController) return;
    // Calling signal.abort() allows us to remove all listeners
    this.abortController = new AbortController();
    const { signal } = this.abortController;

    await super.attach(canvas);
    this._canvas = canvas;

    // Add event listeners to the canvas
    CANVAS_EVENTS.forEach((type) => {
      const listener = this[`on${type}`].bind(this);
      // @ts-ignore - This is a really hard type to make
      this._canvas?.addEventListener(type, listener, { signal });
    });

    // Add event listeners to the document
    DOCUMENT_EVENTS.forEach((type) => {
      const listener = this[`on${type}`].bind(this);
      // @ts-ignore - This is a really hard type to make
      window.document?.addEventListener(type, listener, { signal });
    });

    // When canvas size changes, resize the underlying renderer
    const resizer = new ResizeObserver(([{ contentRect }]) => {
      const dpi = window.devicePixelRatio;
      const { width, height } = contentRect;
      this.resize(width, height, dpi);
    });
    resizer.observe(canvas, { box: "content-box" });
    signal.addEventListener("abort", () => {
      console.log("aborted!");
      resizer.disconnect();
    });

    // Start the animation loop
    const onAnimationFrame: FrameRequestCallback = (dt) => {
      if (signal.aborted) return;
      this.update(dt);
      this.draw(dt);
      requestAnimationFrame(onAnimationFrame);
    };
    requestAnimationFrame(onAnimationFrame);
  }

  /**
   * Removes all event listeners, stops the render loop, and deallocates all the
   * GPU resources of the app, effectively "removing" the app from the canvas.
   */
  unattach() {
    if (!this.abortController) return;
    this.abortController?.abort();
    super.unattach();
  }

  onpointermove(e: PointerEvent) {
    this.handle_mouse_move(e.offsetX, e.offsetY);
  }

  onwheel(e: WheelEvent) {
    this.handle_wheel(e.deltaX, e.deltaY);
    e.stopPropagation();
    e.preventDefault();
  }

  onkeydown(e: KeyboardEvent) {
    if (this.dispatch_key_event(e.code, PressedState.Pressed)) {
      e.stopPropagation();
      e.preventDefault();
    }

    if (this.modifiers.handledEvent(e)) {
      this.handle_modifiers_changed(this.modifiers.value);
    }
  }

  onkeyup(e: KeyboardEvent) {
    if (this.dispatch_key_event(e.code, PressedState.Unpressed)) {
      e.stopPropagation();
      e.preventDefault();
    }

    if (this.modifiers.handledEvent(e)) {
      this.handle_modifiers_changed(this.modifiers.value);
    }
  }

  onmousedown(e: MouseEvent) {
    this.handle_mouse_button(e.button, PressedState.Pressed);
    console.log(this.get_editor_snapshot());
  }

  onmouseup(e: MouseEvent) {
    this.handle_mouse_button(e.button, PressedState.Unpressed);
  }

  private dispatch_key_event(key: string, pressed: PressedState) {
    return this.handle_key(key, pressed);
  }
}
