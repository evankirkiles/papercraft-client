import styles from "./styles.module.scss";

export default function Stats() {
  return (
    <div className={styles.container} aria-label="Toolbar">
      <p>Faces: 30</p>
      <p>Edges: 150</p>
      <p>Vertices: 1,321</p>
    </div>
  );
}
