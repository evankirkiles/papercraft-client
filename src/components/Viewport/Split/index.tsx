import { useEditor } from "@/contexts/EditorContext";
import { useEngine } from "@/contexts/EngineContext";
import { SplitId } from "@paper/core";
import { useEffect, useRef } from "react";
import { DraggableCore } from "react-draggable";

import styles from "./Split.module.scss";

interface SplitProps {
  id: SplitId;
}

const BI_32 = BigInt(32);
function toFFI({ idx, version }: { idx: number; version: number }): bigint {
  return (BigInt(version) << BI_32) | BigInt(idx);
}

export default function Split({ id }: SplitProps) {
  const engine = useEngine();
  const editor = useEditor();
  const container = useRef<HTMLDivElement>(null);
  const slider = useRef<HTMLDivElement>(null);
  const cell1 = useRef<HTMLDivElement>(null);
  const split = editor?.splits[id.idx].value;

  useEffect(() => {
    if (!split || !cell1.current) return;
    cell1.current.style.width = `${split.ratio * 100}%`;
  }, [split]);
  if (!split) return null;

  return (
    <div ref={container} className={styles.container}>
      <div className={styles.cell} ref={cell1}></div>
      <DraggableCore
        nodeRef={slider as React.RefObject<HTMLElement>}
        onStart={() => slider.current?.classList?.add(styles.active)}
        onStop={() => slider.current?.classList?.remove(styles.active)}
        onDrag={(e, data) => {
          if (!slider.current || !cell1.current) return;
          const splitX = data.x;
          const ratio = Math.max(0, Math.min(1, splitX / window.innerWidth));
          cell1.current.style.width = `${ratio * 100}%`;
          engine?.update_split(toFFI(id), ratio);
          e.stopPropagation();
        }}
      >
        <div className={styles.divider} ref={slider} />
      </DraggableCore>
      <div></div>
    </div>
  );
}
