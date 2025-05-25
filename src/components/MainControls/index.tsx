import { useEngineContext } from "@/contexts/EngineContext";
import styles from "./styles.module.scss";
import { SelectionMode } from "@paper/core";

export default function MainControls() {
  const { app } = useEngineContext();

  return (
    <div className={styles.container} aria-label="Main Controls Panel">
      <button className={styles.control_main}>
        <svg
          className={styles.icon}
          width="2em"
          height="2em"
          viewBox="0 0 294 344"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M4 88.0804V256.339L147 339M4 88.0804L147 170.741M4 88.0804L147 5L290 88.0804M147 170.741V339M147 170.741L290 88.0804M147 170.741L180 190L272 138.5L290 88.0804M147 339L290 256.339V88.0804"
            stroke="currentColor"
            strokeWidth="20"
          />
        </svg>
      </button>
      <div className={styles.control_small}>
        <select
          onChange={(e) => {
            if (!app) return;
            const mode = {
              verts: SelectionMode.Vert,
              edges: SelectionMode.Edge,
              faces: SelectionMode.Face,
              pieces: SelectionMode.Piece,
            }[e.target.value];
            if (mode === undefined) return;
            app.set_select_mode(mode);
          }}
        >
          <option value="verts">Verts</option>
          <option value="edges">Edges</option>
          <option value="faces">Faces</option>
          <option value="pieces">Pieces</option>
        </select>
      </div>
      <div className={styles.control_small}></div>
      <div className={styles.control_small}></div>
    </div>
  );
}
