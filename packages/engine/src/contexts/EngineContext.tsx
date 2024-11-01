import { InitOutput } from "crate";
import { createContext, useContext } from "react";

interface EngineContextType {
  engine: InitOutput | null;
}

export const EngineContext = createContext<EngineContextType>({
  engine: null,
});

export const useEngineContext = () => useContext(EngineContext);
