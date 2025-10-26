import { init, SyncConnectionConfig, PaperClient } from "@paperarium/client";
import { useQuery } from "@tanstack/react-query";
import { createContext, PropsWithChildren, useContext } from "react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Spinner } from "@/components/ui/spinner";

export const EngineContext = createContext<PaperClient | undefined>(undefined);
export const useEngine = () => useContext(EngineContext);

export function EngineProvider({ children }: PropsWithChildren) {
  const { data: app, isPending } = useQuery({
    queryKey: ["app"],
    queryFn: () =>
      init().then((output) => {
        output.__wbindgen_start();
        output.install_logging();
        const app = new PaperClient();
        // Connect to multiplayer server (to move out of here, eventually)
        const URI = "ws://localhost:8080";
        const config = new SyncConnectionConfig(URI, "villager");
        app.load_live(config);
        return app;
      }),
    refetchOnReconnect: false,
    refetchOnWindowFocus: false,
  });

  if (isPending) {
    return (
      <div className="flex h-screen w-screen items-center justify-center">
        <Alert className="max-w-md">
          <Spinner />
          <AlertTitle>Loading Engine</AlertTitle>
          <AlertDescription>
            Downloading and initializing the Papercraft engine...
          </AlertDescription>
        </Alert>
      </div>
    );
  }

  return (
    <EngineContext.Provider value={app}>{children}</EngineContext.Provider>
  );
}
