import { LucidePrinter, UserIcon } from "lucide-react";
import { Button } from "@/components/ui/button";

export default function TopRightControls() {
  return (
    <div className="absolute bg-card border border-accent rounded-lg p-2 top-4 right-4 flex items-center gap-3">
      <Button
        variant="ghost"
        size="icon-sm"
        className="rounded-full size-8 border border-accent"
        aria-label="Profile"
        onClick={() => {
          // TODO: Implement profile functionality
          console.log("Profile clicked");
        }}
      >
        <UserIcon />
      </Button>
      <Button
        size="sm"
        onClick={() => {
          // TODO: Implement share functionality
          console.log("Print clicked");
        }}
      >
        <LucidePrinter />
        <span>Print</span>
      </Button>
    </div>
  );
}
