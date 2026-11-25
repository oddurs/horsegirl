import type { ReactNode } from "react";
import styles from "./components.module.css";

// A picture on diagram paper. The picture reads the --d-* tokens.
export function Figure({ caption, children }: { caption: ReactNode; children: ReactNode }) {
  return (
    <figure className={styles.figure}>
      <div className={styles.canvas}>{children}</div>
      <figcaption className={styles.caption}>{caption}</figcaption>
    </figure>
  );
}
