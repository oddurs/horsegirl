import Link from "next/link";
import { loadPages } from "../../lib/docs.ts";

export default function DocsIndex() {
  return (
    <>
      <h1>Documentation</h1>
      <ul>
        {loadPages().map((page) => (
          <li key={page.slug}>
            <Link href={`/docs/${page.slug}/`}>{page.title}</Link>
          </li>
        ))}
      </ul>
    </>
  );
}
