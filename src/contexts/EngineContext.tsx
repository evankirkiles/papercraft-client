import PaperApp from "@/controller";
import init from "@paper/core";
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
      await fetch("/assets/link2.glb")
        .then((res) => {
          if (!res.ok) return;
          return res.arrayBuffer();
        })
        .then((arrayBuffer) => {
          if (!mounted || !arrayBuffer) return;
          app.load_save(new Uint8Array(arrayBuffer));
          console.log("Loaded save file from /public/assets/link2.glb");
        });
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
