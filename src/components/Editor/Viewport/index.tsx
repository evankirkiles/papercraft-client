import { useEditor } from "@/contexts/EditorContext";
import { ViewportId } from "@paper/core";

interface ViewportProps {
  id: ViewportId;
}

export default function Viewport({ id }: ViewportProps) {
  const editor = useEditor();
  const viewport = editor?.viewports[id.idx].value;
  if (!viewport) return null;
  switch (viewport.content) {
  }
}
