import { SettingsIcon } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Switch } from "@/components/ui/switch";
import { useEditor } from "@/contexts/EditorContext";
import { useEngine } from "@/contexts/EngineContext";

interface Dimensions {
  width: number;
  depth: number;
  height: number;
}

const DIMENSION_FIELDS: (keyof Dimensions)[] = ["width", "depth", "height"];

export default function SettingsMenu() {
  const engine = useEngine();
  const editor = useEditor();

  // The model's real-world dimensions (cm), as last read from the Rust side.
  // Editing a field computes a uniform scale factor (newValue / current) and
  // applies it via `scale_mesh`, rather than an arbitrary scale slider.
  const [dims, setDims] = useState<Dimensions | null>(null);
  const [drafts, setDrafts] = useState<Record<keyof Dimensions, string>>({
    width: "",
    depth: "",
    height: "",
  });
  // Which field (if any) the user is actively typing in, so the polling loop
  // below doesn't clobber their in-progress edit.
  const editingField = useRef<keyof Dimensions | null>(null);
  const lastKey = useRef<string>("");

  // The engine reference is stable even when a new document streams in over
  // the websocket (e.g. loading a saved model), so a one-shot effect would
  // only ever see the bounds at mount time. Poll every frame instead, same
  // as BoundsPanel, so dimensions stay in sync once a model finishes loading.
  useEffect(() => {
    if (!engine) return;
    let raf: number;
    const tick = () => {
      try {
        const bounds = engine.get_mesh_bounds();
        const key = `${bounds.width}|${bounds.depth}|${bounds.height}`;
        if (key !== lastKey.current) {
          lastKey.current = key;
          setDims(bounds);
          setDrafts((prev) => {
            const next = { ...prev };
            DIMENSION_FIELDS.forEach((field) => {
              if (editingField.current !== field) {
                next[field] = bounds[field].toFixed(1);
              }
            });
            return next;
          });
        }
      } catch {
        // engine not fully attached yet; ignore, retry next frame
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [engine]);

  const commitDimension = (field: keyof Dimensions) => {
    editingField.current = null;
    if (!engine || !dims) return;
    const next = parseFloat(drafts[field]);
    const current = dims[field];
    if (!Number.isFinite(next) || next <= 0 || current <= 0) {
      setDrafts((d) => ({ ...d, [field]: current.toFixed(1) }));
      return;
    }
    const factor = next / current;
    if (Math.abs(factor - 1) > 1e-4) {
      engine.scale_mesh(factor);
    }
    // The polling loop above will pick up the new bounds on the next frame.
  };

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
            <h3 className="font-semibold mb-1">Model Settings</h3>
            <p className="text-xs text-muted-foreground">
              Configure the model's display and real-world size
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
              <span className="text-muted-foreground">Dimensions (cm)</span>
              <div className="mt-2 grid grid-cols-3 gap-1.5">
                {DIMENSION_FIELDS.map((field) => (
                  <div key={field} className="space-y-0.5">
                    <label
                      htmlFor={`settings-dim-${field}`}
                      className="block text-[10px] uppercase tracking-wide text-muted-foreground/70"
                    >
                      {field}
                    </label>
                    <input
                      id={`settings-dim-${field}`}
                      type="number"
                      min={0}
                      step={0.1}
                      className="w-full rounded-none border bg-background px-1.5 py-1 text-xs tabular-nums focus:outline-none focus:ring-1 focus:ring-ring"
                      value={drafts[field]}
                      disabled={!dims}
                      onFocus={() => {
                        editingField.current = field;
                      }}
                      onChange={(e) =>
                        setDrafts((d) => ({ ...d, [field]: e.target.value }))
                      }
                      onBlur={() => commitDimension(field)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") {
                          e.currentTarget.blur();
                        }
                      }}
                    />
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
