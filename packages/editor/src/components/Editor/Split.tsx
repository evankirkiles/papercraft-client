import { useEditor } from "@/contexts/EditorContext";
import { useEngine } from "@/contexts/EngineContext";
import { SplitId } from "@paperarium/client";
import Node from "./Node";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";

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
      direction={isVertical ? "vertical" : "horizontal"}
      onLayout={(layout) => engine?.update_split(toFFI(id), layout[0] / 100)}
    >
      <ResizablePanel>
        <Node node={split.first} />
      </ResizablePanel>
      <ResizableHandle />
      <ResizablePanel>
        <Node node={split.second} />
      </ResizablePanel>
    </ResizablePanelGroup>
  );
}
