import { SplitId } from "@paperarium/client";

import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { useEditor } from "@/contexts/EditorContext";
import { useEngine } from "@/contexts/EngineContext";

import Node from "./Node";

interface SplitProps {
  id: SplitId;
}

const BI_32 = BigInt(32);
/** Converts a panel split back to its internal rust representation. */
function toFFI({ idx, version }: { idx: number; version: number }): bigint {
  return (BigInt(version) << BI_32) | BigInt(idx);
}

export default function Split({ id }: SplitProps) {
  const engine = useEngine();
  const editor = useEditor();
  const split = editor?.splits[id.idx].value;
  if (!split) return null;
  const isVertical = split.direction === "Vertical";

  return (
    <ResizablePanelGroup
      orientation={isVertical ? "vertical" : "horizontal"}
      onLayoutChange={(layout) => {
        engine?.update_split(toFFI(id), layout[`${id.idx}_0`] / 100);
      }}
    >
      <ResizablePanel id={`${id.idx}_0`}>
        <Node node={split.first} />
      </ResizablePanel>
      <ResizableHandle />
      <ResizablePanel id={`${id.idx}_1`}>
        <Node node={split.second} />
      </ResizablePanel>
    </ResizablePanelGroup>
  );
}
