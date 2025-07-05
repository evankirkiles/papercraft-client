import {
  createContext,
  PropsWithChildren,
  useContext,
  useSyncExternalStore,
} from "react";
import { Editor } from "@paper/core";
import { useEngine } from "./EngineContext";

export const EditorContext = createContext<Editor | undefined>(undefined);
export const useEditor = () => useContext(EditorContext);

export function EditorProvider({ children }: PropsWithChildren) {
  const engine = useEngine();
  const editor = useSyncExternalStore(
    (update) =>
      (engine?.subscribe_to_editor(update) as () => void) || (() => {}),
    () => engine?.editor
  );

  return (
    <EditorContext.Provider value={editor}>{children}</EditorContext.Provider>
  );
}
