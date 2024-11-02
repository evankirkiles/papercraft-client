import { EngineContext } from "@/contexts/EngineContext";
import init, { InitOutput, start } from "crate";
import { useEffect, useState } from "react";

// const CANVAS_ID = "paperarium-engine";
const CANVAS_ID = "canvas";

export function Engine() {
  const [engine, setEngine] = useState<InitOutput | null>(null);
  useEffect(() => {
    let mounted = true;
    init().then((out) => {
      if (!mounted) return;
      setEngine(out);
      start(CANVAS_ID);
    });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <EngineContext.Provider value={{ engine }}>
      <canvas id={CANVAS_ID} width={400} height={400} />
    </EngineContext.Provider>
  );
}
