import { useEffect, useRef } from "react";
import Draggable, { DraggableCore } from "react-draggable";
import styles from "./styles.module.scss";
import { useEngineContext } from "@/contexts/EngineContext";

const CANVAS_ID = "paperarium-engine";

export default function Viewport() {
  const { app } = useEngineContext();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!canvasRef.current || !app) return;
    app.attach(canvasRef.current);
    return () => {};
  }, []);

  return (
    <section className={styles.container} aria-label="Viewport">
      <canvas className={styles.canvas} id={CANVAS_ID} ref={canvasRef} />
      <DraggableCore
        nodeRef={ref as React.RefObject<HTMLElement>}
        onStart={() => {
          canvasRef.current?.classList?.add(styles.inactive);
          ref.current?.classList?.add(styles.active);
        }}
        onStop={() => {
          canvasRef.current?.classList?.remove(styles.inactive);
          ref.current?.classList?.remove(styles.active);
        }}
        onDrag={(e, data) => {
          if (!ref.current) return;
          const splitX = data.x;
          const split = Math.max(0, Math.min(1, splitX / window.innerWidth));
          ref.current.style.left = `${split * 100}%`;
          app?.update_horizontal_split(split);
          e.stopPropagation();
        }}
      >
        <div className={styles.divider} ref={ref} />
      </DraggableCore>
    </section>
  );
}
