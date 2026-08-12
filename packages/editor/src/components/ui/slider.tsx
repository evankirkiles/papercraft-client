import * as SliderPrimitive from "@radix-ui/react-slider"
import * as React from "react"

import { cn } from "@/lib/utils"

function Slider({
  className,
  ...props
}: React.ComponentProps<typeof SliderPrimitive.Root>) {
  return (
    <SliderPrimitive.Root
      data-slot="slider"
      className={cn(
        "relative flex w-full touch-none items-center select-none data-[disabled]:opacity-50",
        className
      )}
      {...props}
    >
      <SliderPrimitive.Track className="bg-input relative h-1.5 w-full grow overflow-hidden rounded-full">
        <SliderPrimitive.Range className="bg-primary absolute h-full" />
      </SliderPrimitive.Track>
      <SliderPrimitive.Thumb className="border-primary bg-background block size-4 shrink-0 rounded-full border shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50" />
    </SliderPrimitive.Root>
  )
}

export { Slider }
