import type { ReactNode } from "react";
import styles from "./components.module.css";

// A wide table scrolls sideways on a phone, so the wrapper is a named region
// that takes keyboard focus.
export function Table({ caption, children }: { caption: string; children: ReactNode }) {
  return (
    // biome-ignore lint/a11y/noNoninteractiveTabindex: a scrolling region must take focus.
    <section className={styles.tableWrap} aria-label={caption} tabIndex={0}>
      <table className={styles.table}>
        <caption>{caption}</caption>
        {children}
      </table>
    </section>
  );
}
