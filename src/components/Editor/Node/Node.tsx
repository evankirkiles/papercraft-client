import { ViewTreeNode } from "@paper/core";
import Split from "../Split/Split";

interface NodeProps {
  node: ViewTreeNode;
}

export default function Node({ node }: NodeProps) {
  switch (true) {
    case "Split" in node:
      return <Split id={node.Split} />;
    case "Viewport" in node:
      node.Viewport;
    default:
      return null;
  }
}
