import { QueryClient, QueryClientProvider } from "react-query";
import Viewport from "./components/Viewport";
import { EngineProvider } from "./contexts/EngineContext";
import { useMemo } from "react";

import styles from "./Engine.module.scss";
import SidePanel from "./components/SidePanel";
import MainControls from "./components/MainControls";
import Stats from "./components/Stats";
import { EditorProvider } from "./contexts/EditorContext";

export function Engine() {
  const client = useMemo(() => new QueryClient(), []);

  return (
    <QueryClientProvider client={client}>
      <EngineProvider>
        <EditorProvider>
          <main className={styles.container}>
            <Viewport />
            <div className={styles.overlay}>
              <MainControls />
              <Stats />
              {/* <Toolbar /> */}
              <SidePanel />
            </div>
          </main>
        </EditorProvider>
      </EngineProvider>
    </QueryClientProvider>
  );
}
