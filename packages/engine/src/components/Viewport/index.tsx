import { useRef } from "react";
import Draggable from "react-draggable";
import styles from "./styles.module.scss";
import { useEngineContext } from "@/contexts/EngineContext";

const CANVAS_ID = "paperarium-engine";

export default function Viewport() {
  const { app } = useEngineContext();
  const ref = useRef<HTMLDivElement>(null);

  return (
    <section className={styles.container} aria-label="Viewport">
      <canvas className={styles.canvas} id={CANVAS_ID} />
      <Draggable
        axis="x"
        nodeRef={ref as React.RefObject<HTMLElement>}
        onDrag={(e, data) => {
          e.stopPropagation();
          const splitX = window.innerWidth / 2 + data.x;
          const split = Math.max(0, Math.min(1, splitX / window.innerWidth));
          app?.resize_viewport(split);
        }}
      >
        <div className={styles.divider} ref={ref} />
      </Draggable>
    </section>
  );
}
