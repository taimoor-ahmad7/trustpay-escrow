import styles from "./Spinner.module.css";

// Spinner shows a small loading indicator while a transaction or fetch is in progress.
export function Spinner() {
  return <span aria-label="Loading" className={styles.spinner} role="status" />;
}
