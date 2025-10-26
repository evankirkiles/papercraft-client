import { CircleHelpIcon } from "lucide-react";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Kbd, KbdGroup } from "@/components/ui/kbd";

export default function HelpButton() {
  return (
    <Popover>
      <PopoverTrigger asChild>
        <button
          className="fixed bottom-4 right-4 z-50 flex size-10 items-center justify-center rounded-full bg-card border shadow-lg hover:bg-accent transition-colors"
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
  );
}
