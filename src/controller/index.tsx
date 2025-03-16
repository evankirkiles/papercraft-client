import {
  App,
  EventHandleSuccess,
  MouseButton,
  NamedKey,
  PressedState,
} from "crates/pp_control/pkg/pp_control";
import { ModifierKeys } from "./modifiers";

const DOCUMENT_EVENTS = [
  "keydown",
  "keyup",
] as const satisfies (keyof HTMLElementEventMap)[];

// TODO: Clean this up
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
    if (this.dispatch_key_event(e.key, PressedState.Pressed)) {
      e.stopPropagation();
      e.preventDefault();
    }

    if (this.modifiers.handledEvent(e)) {
      this.handle_modifiers_changed(this.modifiers.value);
    }
  }

  onkeyup(e: KeyboardEvent) {
    console.log(e);
    if (this.dispatch_key_event(e.key, PressedState.Unpressed)) {
      e.stopPropagation();
      e.preventDefault();
    }

    if (this.modifiers.handledEvent(e)) {
      this.handle_modifiers_changed(this.modifiers.value);
    }
  }

  onmousedown(e: MouseEvent) {
    this.handle_mouse_button(e.button, PressedState.Pressed);
  }

  onmouseup(e: MouseEvent) {
    this.handle_mouse_button(e.button, PressedState.Unpressed);
  }

  /** AN internal wrapper for sending key evnets to the proper Rust callback. */
  private dispatch_key_event(key: string, pressed: PressedState) {
    if (key.length > 1) {
      const val = NAMED_KEY_MAP[key];
      if (val) return this.handle_named_key(val, pressed);
    } else {
      const val = key.charCodeAt(0);
      return this.handle_key(val, pressed);
    }
    return EventHandleSuccess.ContinuePropagation;
  }
}

const NAMED_KEY_MAP: Record<string, NamedKey> = {
  Alt: NamedKey.Alt,
  CapsLock: NamedKey.CapsLock,
  Control: NamedKey.Control,
  Enter: NamedKey.Enter,
  Meta: NamedKey.Meta,
  Redo: NamedKey.Redo,
  Tab: NamedKey.Tab,
  Undo: NamedKey.Undo,
  Escape: NamedKey.Escape,
};
