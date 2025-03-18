import PaperApp from "@/controller";
import { install_logging } from "@paper/core";
import {
  createContext,
  PropsWithChildren,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

interface EngineContextType {
  app: PaperApp | null;
}

export const EngineContext = createContext<EngineContextType>({
  app: null,
});

export const useEngineContext = () => useContext(EngineContext);

install_logging();

export function EngineProvider({ children }: PropsWithChildren) {
  const app = useMemo(() => new PaperApp(), []);
  return (
    <EngineContext.Provider value={{ app }}>{children}</EngineContext.Provider>
  );
}
