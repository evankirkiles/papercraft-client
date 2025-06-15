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
        aria-expanded={expanded}
        className={styles.header}
      >
        HI
      </button>
      <div className={styles.content}>
        Controls (Trackpad-oriented):
        <ul>
          <li>Scroll: Orbit</li>
          <li>CMD + Scroll: Zoom</li>
          <li>SHIFT + Scroll: Pan</li>
          <li>S / Alt+S: Mark cut line</li>
          <li>D: Switch tab edge</li>
          <li>G: Move selected piece (in 2D viewport)</li>
        </ul>
      </div>
    </section>
  );
}
