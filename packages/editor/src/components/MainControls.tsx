import { useEngine } from "@/contexts/EngineContext";
import { SelectionMode } from "@paperarium/client";
import {
  RadioGroupIcon,
  RadioGroupIconItem,
} from "@/components/ui/radio-group-icon";
import { Button } from "@/components/ui/button";

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
      <Button variant="outline" size="icon-xl">
        <svg
          width="2em"
          height="2em"
          viewBox="0 0 294 344"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          className="opacity-70 size-7"
        >
          <path
            d="M4 88.0804V256.339L147 339M4 88.0804L147 170.741M4 88.0804L147 5L290 88.0804M147 170.741V339M147 170.741L290 88.0804M147 170.741L180 190L272 138.5L290 88.0804M147 339L290 256.339V88.0804"
            stroke="currentColor"
            strokeWidth="20"
          />
        </svg>
      </Button>
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
