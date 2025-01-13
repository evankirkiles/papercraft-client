import styles from "./styles.module.scss";

const CANVAS_ID = "paperarium-engine";

export default function Viewport() {
  return (
    <section className={styles.container} aria-label="Viewport">
      <canvas className={styles.canvas} id={CANVAS_ID} />
      <div className={styles.overlay}>
        <div>Vertices: 123 Faces: 123</div>
        <div className={styles.divider} />
      </div>
    </section>
  );
}
