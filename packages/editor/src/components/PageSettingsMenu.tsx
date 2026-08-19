import type { PageSize, PrintLayoutSettings } from "@paperarium/client";
import { FileTextIcon } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Switch } from "@/components/ui/switch";
import { useEditor } from "@/contexts/EditorContext";
import { useEngine } from "@/contexts/EngineContext";

/// Mirrors `PageSize::dimensions` on the Rust side, so the Custom inputs can be
/// seeded with whatever the current preset measures.
const PRESET_DIMENSIONS = {
  A4: { width: 21.0, height: 29.7 },
  Letter: { width: 8.5 * 2.54, height: 11.0 * 2.54 },
} as const;

type PageSizeKind = "A4" | "Letter" | "Custom";

const PAGE_SIZE_KINDS: PageSizeKind[] = ["A4", "Letter", "Custom"];

function kindOf(size: PageSize): PageSizeKind {
  return typeof size === "string" ? size : "Custom";
}

function dimensionsOf(size: PageSize) {
  return typeof size === "string" ? PRESET_DIMENSIONS[size] : size.Custom;
}

/// The numeric fields, keyed by the part of `PrintLayoutSettings` they edit.
type Field = "width" | "height" | "marginX" | "marginY";

const NUMERIC_INPUT_CLASS =
  "w-full rounded-none border bg-background px-1.5 py-1 text-xs tabular-nums focus:outline-none focus:ring-1 focus:ring-ring";

/// Settings for the 2D (cutting / printing) side of the document: whether fold
/// lines are drawn, and the size and margins of the pages the pieces lay out on.
///
/// Fold lines are an editor preference, so they read off the pushed editor
/// snapshot. The page layout lives on the Rust document state instead, which
/// isn't part of that snapshot, so it is polled while the popover is open -
/// which also makes the panel follow an undo, a redo, or a peer's change.
export default function PageSettingsMenu() {
  const engine = useEngine();
  const editor = useEditor();

  const [open, setOpen] = useState(false);
  const [layout, setLayout] = useState<PrintLayoutSettings | null>(null);
  const [drafts, setDrafts] = useState<Record<Field, string>>({
    width: "",
    height: "",
    marginX: "",
    marginY: "",
  });
  // Which field the user is actively typing in, so the poll below doesn't
  // clobber an in-progress edit.
  const editingField = useRef<Field | null>(null);
  const lastKey = useRef<string>("");

  useEffect(() => {
    if (!engine || !open) return;
    let raf: number;
    const tick = () => {
      try {
        const next = engine.get_print_layout();
        const dims = dimensionsOf(next.page_size);
        const key = `${kindOf(next.page_size)}|${dims.width}|${dims.height}|${next.margin_x}|${next.margin_y}`;
        if (key !== lastKey.current) {
          lastKey.current = key;
          setLayout(next);
          const values: Record<Field, number> = {
            width: dims.width,
            height: dims.height,
            marginX: next.margin_x,
            marginY: next.margin_y,
          };
          setDrafts((prev) => {
            const updated = { ...prev };
            (Object.keys(values) as Field[]).forEach((field) => {
              if (editingField.current !== field) {
                updated[field] = values[field].toFixed(2);
              }
            });
            return updated;
          });
        }
      } catch {
        // engine not fully attached yet; ignore, retry next frame
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [engine, open]);

  // Reset the change-key when the popover closes, so reopening re-seeds the
  // drafts even if nothing moved in the meantime.
  useEffect(() => {
    if (!open) lastKey.current = "";
  }, [open]);

  const apply = useCallback(
    (next: PrintLayoutSettings) => {
      engine?.set_print_layout(next);
      // The poll picks the committed value back up on the next frame.
    },
    [engine]
  );

  const selectKind = (kind: PageSizeKind) => {
    if (!layout || kind === kindOf(layout.page_size)) return;
    // Custom starts from whatever the outgoing preset measured, so switching
    // to it is a no-op until the user actually edits a dimension.
    const page_size: PageSize =
      kind === "Custom" ? { Custom: dimensionsOf(layout.page_size) } : kind;
    apply({ ...layout, page_size });
  };

  const commit = (field: Field) => {
    editingField.current = null;
    if (!layout) return;
    const value = parseFloat(drafts[field]);
    const dims = dimensionsOf(layout.page_size);
    const current: Record<Field, number> = {
      width: dims.width,
      height: dims.height,
      marginX: layout.margin_x,
      marginY: layout.margin_y,
    };
    // Page dimensions must be positive; a margin of zero is perfectly valid.
    const min =
      field === "marginX" || field === "marginY" ? 0 : Number.MIN_VALUE;
    if (!Number.isFinite(value) || value < min) {
      setDrafts((d) => ({ ...d, [field]: current[field].toFixed(2) }));
      return;
    }
    if (Math.abs(value - current[field]) < 1e-6) return;

    if (field === "marginX" || field === "marginY") {
      apply({
        ...layout,
        [field === "marginX" ? "margin_x" : "margin_y"]: value,
      });
    } else {
      apply({
        ...layout,
        page_size: { Custom: { ...dims, [field]: value } },
      });
    }
  };

  const numericInput = (field: Field, label: string) => (
    <div className="space-y-0.5">
      <label
        htmlFor={`page-settings-${field}`}
        className="block text-[10px] uppercase tracking-wide text-muted-foreground/70"
      >
        {label}
      </label>
      <input
        id={`page-settings-${field}`}
        type="number"
        min={0}
        step={0.1}
        className={NUMERIC_INPUT_CLASS}
        value={drafts[field]}
        disabled={!layout}
        onFocus={() => {
          editingField.current = field;
        }}
        onChange={(e) => setDrafts((d) => ({ ...d, [field]: e.target.value }))}
        onBlur={() => commit(field)}
        onKeyDown={(e) => {
          if (e.key === "Enter") e.currentTarget.blur();
        }}
      />
    </div>
  );

  const activeKind = layout ? kindOf(layout.page_size) : null;

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          className="flex size-10 items-center justify-center rounded-none bg-card border shadow-lg hover:bg-accent"
          aria-label="Show 2D settings"
        >
          <FileTextIcon className="size-5 text-muted-foreground" />
        </button>
      </PopoverTrigger>
      <PopoverContent side="top" align="end" className="w-64">
        <div className="space-y-3">
          <div>
            <h3 className="font-semibold mb-1">2D Settings</h3>
            <p className="text-xs text-muted-foreground">
              Configure how pieces are laid out and printed
            </p>
          </div>
          <div className="space-y-3 text-sm">
            <label
              htmlFor="page-settings-fold-lines"
              className="flex items-center justify-between cursor-pointer"
            >
              <span className="text-muted-foreground">Fold lines</span>
              <Switch
                id="page-settings-fold-lines"
                checked={editor?.preferences.theme.sizes.fold_lines ?? true}
                onCheckedChange={(checked) => engine?.set_fold_lines(checked)}
              />
            </label>
            <div>
              <span className="text-muted-foreground">Page size</span>
              <div
                role="group"
                aria-label="Page size"
                className="mt-2 grid grid-cols-3 gap-1.5"
              >
                {PAGE_SIZE_KINDS.map((kind) => (
                  <Button
                    key={kind}
                    size="sm"
                    variant={activeKind === kind ? "default" : "outline"}
                    aria-pressed={activeKind === kind}
                    disabled={!layout}
                    onClick={() => selectKind(kind)}
                  >
                    {kind}
                  </Button>
                ))}
              </div>
              {activeKind === "Custom" && (
                <div className="mt-2 grid grid-cols-2 gap-1.5">
                  {numericInput("width", "width")}
                  {numericInput("height", "height")}
                </div>
              )}
            </div>
            <div>
              <span className="text-muted-foreground">Margins (cm)</span>
              <div className="mt-2 grid grid-cols-2 gap-1.5">
                {numericInput("marginX", "x")}
                {numericInput("marginY", "y")}
              </div>
            </div>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
