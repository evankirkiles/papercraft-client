import { SettingsIcon } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { useEditor } from "@/contexts/EditorContext";
import { useEngine } from "@/contexts/EngineContext";

export default function SettingsMenu() {
  const engine = useEngine();
  const editor = useEditor();

  // The model's scale is stored on the Rust side as an absolute value, but
  // `scale_mesh` is incremental, so we track the last-committed absolute
  // value here to compute the factor to apply on each change.
  const [modelScale, setModelScale] = useState(1);
  const lastCommittedScale = useRef(1);

  useEffect(() => {
    if (!engine) return;
    const scale = engine.get_mesh_scale();
    setModelScale(scale);
    lastCommittedScale.current = scale;
  }, [engine]);

  return (
    <Popover>
      <PopoverTrigger asChild>
        <button
          className="flex size-10 items-center justify-center rounded-none bg-card border shadow-lg hover:bg-accent"
          aria-label="Show settings"
        >
          <SettingsIcon className="size-5 text-muted-foreground" />
        </button>
      </PopoverTrigger>
      <PopoverContent side="top" align="end" className="w-64">
        <div className="space-y-3">
          <div>
            <h3 className="font-semibold mb-1">Settings</h3>
            <p className="text-xs text-muted-foreground">
              Configure the editor's viewport and display options
            </p>
          </div>
          <div className="space-y-2 text-sm">
            <label
              htmlFor="settings-xray"
              className="flex items-center justify-between cursor-pointer"
            >
              <span className="text-muted-foreground">X-Ray mode</span>
              <Switch
                id="settings-xray"
                checked={editor?.state.is_xray ?? false}
                onCheckedChange={(checked) => engine?.set_is_xray(checked)}
              />
            </label>
            <div>
              <div className="flex items-center justify-between">
                <label
                  htmlFor="settings-scale"
                  className="text-muted-foreground"
                >
                  Model scale
                </label>
                <span className="text-xs text-muted-foreground tabular-nums">
                  {modelScale.toFixed(2)}x
                </span>
              </div>
              <Slider
                id="settings-scale"
                className="mt-2"
                min={0.1}
                max={5}
                step={0.01}
                value={[modelScale]}
                onValueChange={([value]) => setModelScale(value)}
                onValueCommit={([value]) => {
                  const factor = value / lastCommittedScale.current;
                  lastCommittedScale.current = value;
                  engine?.scale_mesh(factor);
                }}
              />
            </div>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
