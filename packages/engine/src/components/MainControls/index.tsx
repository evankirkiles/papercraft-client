import styles from "./styles.module.scss";

export default function MainControls() {
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
            stroke-width="20"
          />
        </svg>
      </button>
      <div className={styles.control_small}></div>
      <div className={styles.control_small}></div>
      <div className={styles.control_small}></div>
    </div>
  );
}
