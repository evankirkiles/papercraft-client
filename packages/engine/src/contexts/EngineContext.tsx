import init, { InitOutput, begin } from "pp_viewer";
import {
  createContext,
  PropsWithChildren,
  useContext,
  useEffect,
  useState,
} from "react";

interface EngineContextType {
  engine: InitOutput | undefined;
}

export const EngineContext = createContext<EngineContextType>({
  engine: undefined,
});

export const useEngineContext = () => useContext(EngineContext);

export function EngineProvider({ children }: PropsWithChildren) {
  const [engine, setEngine] = useState<InitOutput | undefined>(undefined);
  useEffect(() => {
    let mounted = true;
    init().then((output) => {
      if (!mounted) return;
      begin();
      setEngine(output);
    });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <EngineContext.Provider value={{ engine }}>
      {children}
    </EngineContext.Provider>
  );
}
