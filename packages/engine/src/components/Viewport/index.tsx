import styles from "./styles.module.scss";

const CANVAS_ID = "paperarium-engine";

export default function Viewport() {
  return (
    <section className={styles.container} aria-label="Viewport">
      <canvas className={styles.canvas} id={CANVAS_ID} />
      <div className={styles.divider} />
    </section>
  );
}
