import { useEffect, useRef } from "react";
import { useEngine } from "@/contexts/EngineContext";
import { useEditor } from "@/contexts/EditorContext";
import Node from "./Node";

const CANVAS_ID = "paperarium-engine";

export default function Viewport() {
  const engine = useEngine();
  const editor = useEditor();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    if (!canvasRef.current || !engine) return;
    engine.attach(canvasRef.current);
    return () => {};
  }, [engine]);

  return (
    <section
      className="size-full relative overflow-hidden"
      aria-label="Viewport"
    >
      <canvas
        className="size-full absolute z-0 outline-none cursor-crosshair"
        id={CANVAS_ID}
        ref={canvasRef}
      />
      {editor && <Node node={editor.root_node} />}
    </section>
  );
}
