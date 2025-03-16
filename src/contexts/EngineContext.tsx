import PaperApp from "@/controller";
import init, { InitOutput, install_logging } from "pp_control";
import {
  createContext,
  PropsWithChildren,
  useContext,
  useEffect,
  useState,
} from "react";

interface EngineContextType {
  engine: InitOutput | undefined;
  app: PaperApp | undefined;
}

export const EngineContext = createContext<EngineContextType>({
  engine: undefined,
  app: undefined,
});

export const useEngineContext = () => useContext(EngineContext);

export function EngineProvider({ children }: PropsWithChildren) {
  const [app, setApp] = useState<PaperApp | undefined>(undefined);
  const [engine, setEngine] = useState<InitOutput | undefined>(undefined);
  useEffect(() => {
    let mounted = true;
    init().then(async (output) => {
      if (!mounted) return;
      install_logging();
      const app = new PaperApp();
      const canvas = document.getElementById("paperarium-engine");
      if (!canvas) throw new Error("missing canvas...");
      await app.attach(canvas as HTMLCanvasElement);
      setApp(app);
      setEngine(output);
    });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <EngineContext.Provider value={{ app, engine }}>
      {children}
    </EngineContext.Provider>
  );
}
