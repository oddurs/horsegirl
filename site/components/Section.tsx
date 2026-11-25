import type { ReactNode } from "react";
import styles from "./components.module.css";

export function Section({
  id,
  eyebrow,
  title,
  lede,
  children,
}: {
  id: string;
  eyebrow?: string;
  title: string;
  lede?: ReactNode;
  children?: ReactNode;
}) {
  return (
    <section id={id} className={styles.section} aria-labelledby={`${id}-title`}>
      <div className={styles.sectionHead}>
        {eyebrow && <p className={styles.eyebrow}>{eyebrow}</p>}
        <h2 id={`${id}-title`}>{title}</h2>
        {lede && <p className={styles.lede}>{lede}</p>}
      </div>
      {children}
    </section>
  );
}
