import { useEffect, useRef, useState } from "react";

import { useEngine } from "@/contexts/EngineContext";

interface Bounds {
  width: number;
  depth: number;
  height: number;
}

// Switch to meters once the largest dimension makes centimeters unwieldy.
const METERS_THRESHOLD_CM = 100;

function formatBounds({ width, depth, height }: Bounds): string {
  const maxDim = Math.max(width, depth, height);
  if (maxDim >= METERS_THRESHOLD_CM) {
    const toM = (cm: number) => (cm / 100).toFixed(2);
    return `${toM(width)} × ${toM(depth)} × ${toM(height)} m`;
  }
  const toCm = (cm: number) => cm.toFixed(1);
  return `${toCm(width)} × ${toCm(depth)} × ${toCm(height)} cm`;
}

export default function BoundsPanel() {
  const engine = useEngine();
  const [bounds, setBounds] = useState<Bounds | null>(null);
  const lastKey = useRef<string>("");

  useEffect(() => {
    if (!engine) return;
    let raf: number;
    const tick = () => {
      // `engine.attach()` allocates the GPU renderer asynchronously; until it
      // resolves, calling any other method on the same wasm object races
      // wasm-bindgen's reentrancy guard. Just retry next frame.
      try {
        const b = engine.get_mesh_bounds();
        const key = `${b.width}|${b.depth}|${b.height}`;
        if (key !== lastKey.current) {
          lastKey.current = key;
          setBounds(b);
        }
      } catch {
        // ignore, retry next frame
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [engine]);

  if (!bounds || (bounds.width === 0 && bounds.depth === 0 && bounds.height === 0)) {
    return null;
  }

  return (
    <div className="fixed bottom-16 right-4 z-50 px-3 py-2 rounded-none bg-card border shadow-lg text-xs text-muted-foreground tabular-nums">
      {formatBounds(bounds)}
    </div>
  );
}
