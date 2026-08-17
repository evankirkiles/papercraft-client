import { SelectionMode, SelectTool } from "@paperarium/client";

import {
  RadioGroupIcon,
  RadioGroupIconItem,
} from "@/components/ui/radio-group-icon";
import { useEditor } from "@/contexts/EditorContext";
import { useEngine } from "@/contexts/EngineContext";

export default function MainControls() {
  const engine = useEngine();
  const editor = useEditor();

  const handleSelectionModeChange = (value: string) => {
    if (!engine) return;
    const modes: Record<string, SelectionMode> = {
      verts: SelectionMode.Vert,
      edges: SelectionMode.Edge,
      faces: SelectionMode.Face,
      pieces: SelectionMode.Piece,
    };
    engine.set_select_mode(modes[value]);
  };

  const handleSelectToolChange = (value: string) => {
    if (!engine) return;
    engine.set_select_tool(
      value === "paint" ? SelectTool.Paint : SelectTool.Box
    );
  };

  // The editor snapshot is serde-serialized, so enums arrive as their variant
  // name rather than the numeric value the generated .d.ts declares.
  const selectTool =
    (editor?.state.select_tool as unknown as string) === "Paint"
      ? "paint"
      : "box";

  return (
    <div
      className="absolute top-4 left-4 flex flex-col items-start gap-2"
      aria-label="Main Controls Panel"
    >
      <RadioGroupIcon
        defaultValue="verts"
        onValueChange={handleSelectionModeChange}
      >
        <RadioGroupIconItem value="verts" aria-label="Select vertices">
          <svg viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="3" fill="currentColor" />
          </svg>
        </RadioGroupIconItem>
        <RadioGroupIconItem value="edges" aria-label="Select edges">
          <svg viewBox="0 0 24 24">
            <line
              x1="4"
              y1="12"
              x2="20"
              y2="12"
              stroke="currentColor"
              strokeWidth="3"
            />
          </svg>
        </RadioGroupIconItem>
        <RadioGroupIconItem value="faces" aria-label="Select faces">
          <svg viewBox="0 0 24 24">
            <polygon points="12,2 22,20 2,20" fill="currentColor" />
          </svg>
        </RadioGroupIconItem>
        <RadioGroupIconItem value="pieces" aria-label="Select pieces">
          <svg viewBox="0 0 24 24">
            <rect x="3" y="3" width="8" height="8" fill="currentColor" />
            <rect x="13" y="3" width="8" height="8" fill="currentColor" />
            <rect x="3" y="13" width="8" height="8" fill="currentColor" />
            <rect x="13" y="13" width="8" height="8" fill="currentColor" />
          </svg>
        </RadioGroupIconItem>
      </RadioGroupIcon>
      <RadioGroupIcon value={selectTool} onValueChange={handleSelectToolChange}>
        <RadioGroupIconItem value="box" aria-label="Box select">
          <svg viewBox="0 0 24 24">
            <rect
              x="3.5"
              y="3.5"
              width="17"
              height="17"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeDasharray="4 3"
            />
          </svg>
        </RadioGroupIconItem>
        <RadioGroupIconItem value="paint" aria-label="Paint select">
          <svg viewBox="0 0 24 24">
            <circle
              cx="12"
              cy="12"
              r="8.5"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeDasharray="3 3"
            />
            <circle cx="12" cy="12" r="3" fill="currentColor" />
          </svg>
        </RadioGroupIconItem>
      </RadioGroupIcon>
    </div>
  );
}
