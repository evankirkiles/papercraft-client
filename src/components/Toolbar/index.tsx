import { useContext } from "react";
import styles from "./styles.module.scss";
import { useEngine } from "@/contexts/EngineContext";

export default function Toolbar() {
  return (
    <div className={styles.container} aria-label="Toolbar">
      <button className={styles.control_main}>Toolbar</button>
    </div>
  );
}
