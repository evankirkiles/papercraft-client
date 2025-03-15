import init, { InitOutput, App } from "pp_control2";
import {
  createContext,
  PropsWithChildren,
  useContext,
  useEffect,
  useState,
} from "react";

interface EngineContextType {
  engine: InitOutput | undefined;
  app: App | undefined;
}

export const EngineContext = createContext<EngineContextType>({
  engine: undefined,
  app: undefined,
});

export const useEngineContext = () => useContext(EngineContext);

export function EngineProvider({ children }: PropsWithChildren) {
  const [app, setApp] = useState<App | undefined>(undefined);
  const [engine, setEngine] = useState<InitOutput | undefined>(undefined);
  useEffect(() => {
    let mounted = true;
    init().then(async (output) => {
      if (!mounted) return;
      const app = new App("paperarium-engine");
      await app.begin();
      const draw = () => {
        app.draw();
        requestAnimationFrame(draw);
      };
      draw();
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
