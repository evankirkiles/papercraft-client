import { SelectionMode } from "@paperarium/client";

import {
  RadioGroupIcon,
  RadioGroupIconItem,
} from "@/components/ui/radio-group-icon";
import { useEngine } from "@/contexts/EngineContext";

export default function MainControls() {
  const engine = useEngine();

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

  return (
    <div
      className="absolute top-4 left-4 flex gap-2"
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
    </div>
  );
}
