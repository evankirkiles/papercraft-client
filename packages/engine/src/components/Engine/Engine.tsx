import { EngineContext } from "@/contexts/EngineContext";
import init, { InitOutput, begin } from "pp_viewer";
import { useEffect, useState } from "react";

const CANVAS_ID = "paperarium-engine";

export function Engine() {
  const [engine, setEngine] = useState<InitOutput | null>(null);
  useEffect(() => {
    let mounted = true;
    init().then((out) => {
      if (!mounted) return;
      setEngine(out);
      begin();
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
