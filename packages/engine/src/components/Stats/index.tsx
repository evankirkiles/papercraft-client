import styles from "./styles.module.scss";

export default function Stats() {
  return (
    <div className={styles.container} aria-label="Toolbar">
      <p>pp_viewer v0.0.1</p>
      <p>{new Date().toLocaleString()}</p>
    </div>
  );
}
