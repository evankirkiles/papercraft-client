import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useMemo } from "react";

import BoundsPanel from "./components/BoundsPanel";
import Viewport from "./components/Editor";
import HelpButton from "./components/HelpButton";
import MainControls from "./components/MainControls";
import { EditorProvider } from "./contexts/EditorContext";
import { EngineProvider } from "./contexts/EngineContext";
import { ThemeProvider } from "./contexts/theme";

export function Engine() {
  const client = useMemo(() => new QueryClient(), []);

  return (
    <QueryClientProvider client={client}>
      <ThemeProvider defaultTheme="dark">
        <EngineProvider>
          <EditorProvider>
            <section className="size-full border rounded-lg overflow-hidden">
              <Viewport />
              <div className="absolute inset-0 p-4 grid pointer-events-none *:pointer-events-auto">
                <MainControls />
              </div>
              <HelpButton />
              <BoundsPanel />
            </section>
          </EditorProvider>
        </EngineProvider>
      </ThemeProvider>
    </QueryClientProvider>
  );
}
