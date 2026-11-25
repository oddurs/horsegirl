import type { ReactNode } from "react";
import styles from "./components.module.css";
import { type NavLink, SiteHeader } from "./SiteHeader";

// The page shell: a skip link, the sticky header and one column of content.
export function Shell({ nav, children }: { nav: NavLink[]; children: ReactNode }) {
  return (
    <>
      <a className={styles.skip} href="#main">
        Skip to content
      </a>
      <SiteHeader nav={nav} />
      <main id="main" className={styles.main}>
        {children}
      </main>
    </>
  );
}
