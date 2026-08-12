import { SettingsIcon } from "lucide-react";

import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Switch } from "@/components/ui/switch";
import { useEditor } from "@/contexts/EditorContext";
import { useEngine } from "@/contexts/EngineContext";

export default function SettingsMenu() {
  const engine = useEngine();
  const editor = useEditor();

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
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
