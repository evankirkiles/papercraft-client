import { QueryClient, QueryClientProvider } from "react-query";
import Viewport from "./components/Viewport";
import { EngineProvider } from "./contexts/EngineContext";
import { useMemo } from "react";

import styles from "./Engine.module.scss";
import SidePanel from "./components/SidePanel";

export function Engine() {
  const client = useMemo(() => new QueryClient(), []);

  return (
    <QueryClientProvider client={client}>
      <EngineProvider>
        <main className={styles.container}>
          <Viewport />
          <SidePanel />
        </main>
      </EngineProvider>
    </QueryClientProvider>
  );
}
