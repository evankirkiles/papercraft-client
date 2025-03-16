import { App } from "crates/pp_control2/pkg/pp_control2";
import { ModifierKeys } from "./modifiers";

const DOCUMENT_EVENTS = [
  "keydown",
  "keyup",
] as const satisfies (keyof HTMLElementEventMap)[];

// TODO: Clean this up
const CANVAS_EVENTS = [
  "wheel",
  "pointermove",
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

  /**
   * Attachs the app to an HTML Canvas.
   *
   * On the Rust side, this retrieves the canvas's GPU context and allocates
   * GPU resources. On the JS side, this attaches listeners to the canvas and
   * to the window for controlling the Rust app.
   */
  async attach(canvas: HTMLCanvasElement) {
    await super.attach(canvas);
    this._canvas = canvas;

    // Calling signal.abort() allows us to remove all listeners
    const controller = new AbortController();
    const { signal } = controller;

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
    signal.addEventListener("abort", () => resizer.disconnect());

    // Start the animation loop
    const onAnimationFrame: FrameRequestCallback = (dt) => {
      if (signal.aborted) return;
      this.draw(dt);
      requestAnimationFrame(onAnimationFrame);
    };
    requestAnimationFrame(onAnimationFrame);

    this.abortController = controller;
  }

  /**
   * Removes all event listeners, stops the render loop, and deallocates all the
   * GPU resources of the app, effectively "removing" the app from the canvas.
   */
  public unattach() {
    if (!this.abortController) return;
    this.abortController.abort();
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
    if (this.modifiers.handledEvent(e)) {
      this.handle_modifiers_changed(this.modifiers.value);
      return;
    }
  }

  onkeyup(e: KeyboardEvent) {
    if (this.modifiers.handledEvent(e)) {
      this.handle_modifiers_changed(this.modifiers.value);
      return;
    }
  }
}
