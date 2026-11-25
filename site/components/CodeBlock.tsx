import type { ReactNode } from "react";
import styles from "./components.module.css";

// A long line scrolls inside the block, so the block takes keyboard focus:
// without it a keyboard user cannot reach the end of the line.
export function CodeBlock({ file, children }: { file?: string; children: ReactNode }) {
  return (
    <div className={styles.code}>
      {file && <span className={styles.file}>{file}</span>}
      {/* biome-ignore lint/a11y/noNoninteractiveTabindex: a scrolling region must take focus. */}
      <pre tabIndex={0}>
        <code>{children}</code>
      </pre>
    </div>
  );
}
