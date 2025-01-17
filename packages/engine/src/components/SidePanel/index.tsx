import { useState } from "react";
import styles from "./styles.module.scss";

export default function SidePanel() {
  const [expanded, setExpanded] = useState(false);
  return (
    <section
      className={styles.container}
      aria-label="Side Panel"
      style={{ height: expanded ? "100%" : undefined }}
    >
      <button
        onClick={() => setExpanded((prev) => !prev)}
        area-expanded={expanded}
        className={styles.header}
      >
        HI
      </button>
    </section>
  );
}
