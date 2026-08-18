import { CircleHelpIcon, LucideLoader2, LucidePrinter } from "lucide-react";
import { useState } from "react";

import SettingsMenu from "@/components/SettingsMenu";
import { Button } from "@/components/ui/button";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { useEngine } from "@/contexts/EngineContext";

export default function HelpButton() {
  const engine = useEngine();
  // A print run renders one page per frame, so it takes a moment on a large
  // layout. Block a second run rather than letting it reject.
  const [isPrinting, setIsPrinting] = useState(false);

  const handlePrint = async () => {
    if (!engine || isPrinting) return;
    setIsPrinting(true);
    try {
      await engine.print();
    } catch (error) {
      console.error("Failed to print the page layout:", error);
    } finally {
      setIsPrinting(false);
    }
  };

  return (
    <div className="fixed bottom-4 right-4 z-50 flex items-center gap-2">
      <SettingsMenu />
      <Button size="sm" onClick={handlePrint} disabled={!engine || isPrinting}>
        {isPrinting ? (
          <LucideLoader2 className="animate-spin" />
        ) : (
          <LucidePrinter />
        )}
        <span>{isPrinting ? "Printing…" : "Print"}</span>
      </Button>
      <Popover>
        <PopoverTrigger asChild>
          <button
            className="flex size-10 items-center justify-center rounded-none bg-card border shadow-lg hover:bg-accent"
            aria-label="Show controls"
          >
            <CircleHelpIcon className="size-5 text-muted-foreground" />
          </button>
        </PopoverTrigger>
        <PopoverContent side="top" align="end" className="w-72">
          <div className="space-y-3">
            <div>
              <h3 className="font-semibold mb-1">Controls</h3>
              <p className="text-xs text-muted-foreground">
                Trackpad-oriented keyboard shortcuts
              </p>
            </div>
            <div className="space-y-2 text-sm">
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Orbit</span>
                <Kbd>Scroll</Kbd>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Zoom</span>
                <KbdGroup>
                  <Kbd>⌘</Kbd>
                  <span className="text-muted-foreground">+</span>
                  <Kbd>Scroll</Kbd>
                </KbdGroup>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Pan</span>
                <KbdGroup>
                  <Kbd>⇧</Kbd>
                  <span className="text-muted-foreground">+</span>
                  <Kbd>Scroll</Kbd>
                </KbdGroup>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Toggle X-Ray Mode</span>
                <Kbd>⇥</Kbd>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Mark cut line</span>
                <KbdGroup>
                  <Kbd>S</Kbd>
                  <span className="text-muted-foreground">/</span>
                  <Kbd>⌥</Kbd>
                  <span className="text-muted-foreground">+</span>
                  <Kbd>S</Kbd>
                </KbdGroup>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Switch tab edge</span>
                <Kbd>D</Kbd>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Move piece (2D)</span>
                <Kbd>G</Kbd>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Fold/Unfold tween</span>
                <KbdGroup>
                  <Kbd>⌃</Kbd>
                  <span className="text-muted-foreground">+</span>
                  <Kbd>Scroll</Kbd>
                </KbdGroup>
              </div>
            </div>
          </div>
        </PopoverContent>
      </Popover>
    </div>
  );
}
