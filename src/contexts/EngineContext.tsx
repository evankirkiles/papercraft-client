import PaperApp from "@/controller";
import init, { SyncConnectionConfig } from "@paper/core";
import {
  createContext,
  PropsWithChildren,
  useContext,
  useEffect,
  useState,
} from "react";

export const EngineContext = createContext<PaperApp | undefined>(undefined);
export const useEngine = () => useContext(EngineContext);

export function EngineProvider({ children }: PropsWithChildren) {
  const [app, setApp] = useState<PaperApp | undefined>(undefined);

  // Load the app in on startup
  useEffect(() => {
    let mounted = true;
    init().then(async (output) => {
      output.__wbindgen_start();
      output.install_logging();
      if (!mounted) return;
      const app = new PaperApp();

      // Connect to multiplayer server
      const URI = "ws://localhost:8080";
      const config = new SyncConnectionConfig(URI, "CesiumMan");
      app.load_live(config);
      console.log(`Connected to multiplayer server at ${URI}`);

      setApp(app);
    });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <EngineContext.Provider value={app}>{children}</EngineContext.Provider>
  );
}
