import { notFound } from "next/navigation";
import { loadPages, renderPages } from "../../../lib/docs.ts";

export const dynamicParams = false;

export function generateStaticParams() {
  return loadPages().map((page) => ({ slug: page.slug.split("/") }));
}

export default async function DocPage({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params;
  const page = renderPages(loadPages()).find((p) => p.slug === slug.join("/"));
  if (!page) notFound();
  // biome-ignore lint/security/noDangerouslySetInnerHtml: the HTML comes from the repository's own Markdown files, which are trusted input.
  return <article dangerouslySetInnerHTML={{ __html: page.html }} />;
}
