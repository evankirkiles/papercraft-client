import * as React from "react";
import * as RadioGroupPrimitive from "@radix-ui/react-radio-group";

import { cn } from "@/lib/utils";

function RadioGroupIcon({
  className,
  ...props
}: React.ComponentProps<typeof RadioGroupPrimitive.Root>) {
  return (
    <RadioGroupPrimitive.Root
      data-slot="radio-group-icon"
      className={cn(
        "flex bg-background border border-border rounded-lg overflow-visible shadow-sm h-fit",
        className
      )}
      {...props}
    />
  );
}

function RadioGroupIconItem({
  className,
  children,
  ...props
}: React.ComponentProps<typeof RadioGroupPrimitive.Item>) {
  return (
    <RadioGroupPrimitive.Item
      data-slot="radio-group-icon-item"
      className={cn(
        "flex items-center justify-center p-2 cursor-pointer transition-all border-r border-border last:border-r-0",
        "first:rounded-l-lg last:rounded-r-lg",
        "hover:bg-muted/50",
        "data-[state=checked]:bg-muted data-[state=checked]:text-foreground",
        "outline-none focus-visible:ring-ring/50 focus-visible:ring-[3px]",
        "disabled:cursor-not-allowed disabled:opacity-50",
        className
      )}
      {...props}
    >
      <div className="w-4 aspect-square opacity-80">{children}</div>
    </RadioGroupPrimitive.Item>
  );
}

export { RadioGroupIcon, RadioGroupIconItem };
