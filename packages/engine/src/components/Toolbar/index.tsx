import { useContext } from "react";
import styles from "./styles.module.scss";
import { useEngineContext } from "@/contexts/EngineContext";

export default function Toolbar() {
  const { app } = useEngineContext();

  return (
    <div className={styles.container} aria-label="Toolbar">
      <button
        className={styles.control_main}
        onClick={() => {
          console.log("APP RUNNING FN", app?.hi());
        }}
      >
        Toolbar
      </button>
    </div>
  );
}
