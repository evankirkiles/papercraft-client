import { useEngine } from "@/contexts/EngineContext";
import styles from "./styles.module.scss";
import { SelectionMode } from "@paperarium/client";

export default function MainControls() {
  const engine = useEngine();

  return (
    <div
      className="absolute top-4 left-4 flex gap-2"
      aria-label="Main Controls Panel"
    >
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
      <div className={styles.select_mode_group}>
        <input
          type="radio"
          id="verts"
          name="select_mode"
          value="verts"
          className={styles.select_mode_radio}
          defaultChecked
          onChange={() => engine?.set_select_mode(SelectionMode.Vert)}
        />
        <label htmlFor="verts" className={styles.select_mode_label}>
          <svg className={styles.select_mode_icon} viewBox="0 0 24 24">
            <circle cx="12" cy="12" r="3" fill="currentColor" />
          </svg>
        </label>
        <input
          type="radio"
          id="edges"
          name="select_mode"
          value="edges"
          className={styles.select_mode_radio}
          onChange={() => engine?.set_select_mode(SelectionMode.Edge)}
        />
        <label htmlFor="edges" className={styles.select_mode_label}>
          <svg className={styles.select_mode_icon} viewBox="0 0 24 24">
            <line
              x1="4"
              y1="12"
              x2="20"
              y2="12"
              stroke="currentColor"
              strokeWidth="3"
            />
          </svg>
        </label>

        <input
          type="radio"
          id="faces"
          name="select_mode"
          value="faces"
          className={styles.select_mode_radio}
          onChange={() => {
            if (!engine) return;
            engine.set_select_mode(SelectionMode.Face);
          }}
        />
        <label htmlFor="faces" className={styles.select_mode_label}>
          <svg className={styles.select_mode_icon} viewBox="0 0 24 24">
            <polygon points="12,2 22,20 2,20" fill="currentColor" />
          </svg>
        </label>

        <input
          type="radio"
          id="pieces"
          name="select_mode"
          value="pieces"
          className={styles.select_mode_radio}
          onChange={() => {
            if (!engine) return;
            engine.set_select_mode(SelectionMode.Piece);
          }}
        />
        <label htmlFor="pieces" className={styles.select_mode_label}>
          <svg className={styles.select_mode_icon} viewBox="0 0 24 24">
            <rect x="3" y="3" width="8" height="8" fill="currentColor" />
            <rect x="13" y="3" width="8" height="8" fill="currentColor" />
            <rect x="3" y="13" width="8" height="8" fill="currentColor" />
            <rect x="13" y="13" width="8" height="8" fill="currentColor" />
          </svg>
        </label>
      </div>
    </div>
  );
}
