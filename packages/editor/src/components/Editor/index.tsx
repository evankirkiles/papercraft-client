import { useEffect, useRef } from "react";
import styles from "./styles.module.scss";
import { useEngine } from "@/contexts/EngineContext";
import { useEditor } from "@/contexts/EditorContext";
import Node from "./Node/Node";

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
    <section className={styles.container} aria-label="Viewport">
      <canvas className={styles.canvas} id={CANVAS_ID} ref={canvasRef} />
      {editor && <Node node={editor.root_node} />}
    </section>
  );
}
