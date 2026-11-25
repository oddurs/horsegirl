import Link from "next/link";
import type { ReactNode } from "react";
import { loadPages } from "../../lib/docs.ts";
import "./docs.css";

export default function DocsLayout({ children }: { children: ReactNode }) {
  const pages = loadPages();
  return (
    <div className="docs">
      <nav aria-label="Documentation">
        <ul>
          {pages.map((page) => (
            <li key={page.slug}>
              <Link href={`/docs/${page.slug}/`}>{page.title}</Link>
            </li>
          ))}
        </ul>
      </nav>
      {/* The root Shell already holds the one <main>, so the content is a plain div. */}
      <div>{children}</div>
    </div>
  );
}
