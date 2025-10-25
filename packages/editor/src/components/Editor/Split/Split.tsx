import { useEditor } from "@/contexts/EditorContext";
import { useEngine } from "@/contexts/EngineContext";
import { SplitId } from "@paperarium/client";
import { useEffect, useRef } from "react";
import { DraggableCore } from "react-draggable";
import cn from "classnames";

import styles from "./Split.module.scss";
import Node from "../Node/Node";

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
    if (isVertical) {
      cell1.current.style.height = `${split.ratio * 100}%`;
    } else {
      cell1.current.style.width = `${split.ratio * 100}%`;
    }
  }, [split]);
  if (!split) return null;
  const isVertical = split.direction === "Vertical";

  return (
    <div
      ref={container}
      className={cn(styles.container, { [styles.vertical]: isVertical })}
    >
      <div className={styles.cell} ref={cell1}>
        <Node node={split.first} />
      </div>
      <DraggableCore
        nodeRef={slider as React.RefObject<HTMLElement>}
        onStart={() => slider.current?.classList?.add(styles.active)}
        onStop={() => slider.current?.classList?.remove(styles.active)}
        onDrag={(e, data) => {
          if (!slider.current || !cell1.current || !container.current) return;
          const bounds = container.current.getBoundingClientRect();
          if (isVertical) {
            const proportion = Math.abs((data.y - bounds.y) / bounds.height);
            const ratio = Math.max(0, Math.min(1, proportion));
            cell1.current.style.height = `${ratio * 100}%`;
            engine?.update_split(toFFI(id), ratio);
          } else {
            const proportion = Math.abs((data.x - bounds.x) / bounds.width);
            const ratio = Math.max(0, Math.min(1, proportion));
            cell1.current.style.width = `${ratio * 100}%`;
            engine?.update_split(toFFI(id), ratio);
          }
          e.stopPropagation();
        }}
      >
        <div
          ref={slider}
          className={cn(styles.divider, { [styles.vertical]: isVertical })}
        />
      </DraggableCore>
      <div className={styles.cell}>
        <Node node={split.second} />
      </div>
    </div>
  );
}
