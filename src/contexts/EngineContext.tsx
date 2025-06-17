import PaperApp from "@/controller";
import init from "@paper/core";
import {
  createContext,
  PropsWithChildren,
  useContext,
  useEffect,
  useState,
} from "react";

interface EngineContextType {
  app: PaperApp | null;
}

export const EngineContext = createContext<EngineContextType>({
  app: null,
});

export const useEngineContext = () => useContext(EngineContext);

export function EngineProvider({ children }: PropsWithChildren) {
  const [app, setApp] = useState<PaperApp | null>(null);
  useEffect(() => {
    let mounted = true;
    init().then((output) => {
      output.__wbindgen_start();
      output.install_logging();
      if (!mounted) return;
      setApp(new PaperApp());
    });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <EngineContext.Provider value={{ app }}>{children}</EngineContext.Provider>
  );
}
