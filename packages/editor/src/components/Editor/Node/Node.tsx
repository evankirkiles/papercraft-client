import { ViewTreeNode } from "@paperarium/client";
import Split from "../Split/Split";

interface NodeProps {
  node: ViewTreeNode;
}

export default function Node({ node }: NodeProps) {
  switch (true) {
    case "Split" in node:
      return <Split id={node.Split} />;
    case "Viewport" in node:
    default:
      return null;
  }
}
