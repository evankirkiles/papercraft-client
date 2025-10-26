import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import Viewport from "./components/Editor";
import { EngineProvider } from "./contexts/EngineContext";
import MainControls from "./components/MainControls";
import Stats from "./components/Stats";
import HelpButton from "./components/HelpButton";
import TopRightControls from "./components/TopRightControls";
import { EditorProvider } from "./contexts/EditorContext";
import { ThemeProvider } from "./contexts/theme";
import { useMemo } from "react";

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
                <Stats />
              </div>
              <TopRightControls />
              <HelpButton />
            </section>
          </EditorProvider>
        </EngineProvider>
      </ThemeProvider>
    </QueryClientProvider>
  );
}
