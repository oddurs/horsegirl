import Link from "next/link";
import styles from "./components.module.css";

export type NavLink = { href: string; label: string };

// The rosette mark from the concept page. The link text names it, so the
// picture is hidden from a screen reader.
function Mark() {
  return (
    <svg viewBox="0 0 80 110" aria-hidden="true" focusable="false">
      <polygon points="26,62 16,106 30,96 38,108 40,66" fill="var(--accent)" />
      <polygon points="54,62 64,106 50,96 42,108 40,66" fill="var(--accent)" />
      <circle
        cx="40"
        cy="40"
        r="32"
        fill="none"
        stroke="var(--accent)"
        strokeWidth="12"
        strokeDasharray="5 3"
      />
      <circle cx="40" cy="40" r="25" fill="var(--ground)" stroke="var(--accent)" strokeWidth="2" />
      <path d="M40 28V38H29V50M40 38H51V50" fill="none" stroke="var(--ink)" strokeWidth="2" />
      <circle cx="40" cy="28" r="4" fill="var(--ink)" />
      <circle cx="29" cy="51" r="4" fill="var(--ink)" />
      <circle cx="51" cy="51" r="4" fill="var(--accent)" />
    </svg>
  );
}

export function SiteHeader({ nav }: { nav: NavLink[] }) {
  return (
    <header className={styles.header}>
      <div className={styles.headerInner}>
        <Link className={styles.brand} href="/">
          <Mark />
          <span className={styles.wordmark}>horsegirl</span>
        </Link>
        <nav className={styles.nav} aria-label="Site">
          {nav.map(({ href, label }) =>
            href.startsWith("/") ? (
              <Link key={href} href={href}>
                {label}
              </Link>
            ) : (
              <a key={href} href={href}>
                {label}
              </a>
            ),
          )}
        </nav>
      </div>
    </header>
  );
}
