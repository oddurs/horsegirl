// Turns the Markdown files under docs/ into site pages. The files stay the only
// copy: a new file becomes a page with no other change. A link that names a
// missing page or heading throws, so the build fails.
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, posix, relative, sep } from "node:path";
import { Marked, type Token, type Tokens } from "marked";
import { basePath } from "./base.ts";
import { escapeHtml, highlightHorsegirl } from "./highlight.ts";

// docs/concept is a prototype, not documentation.
const excluded = new Set(["concept"]);

export type Page = { slug: string; file: string; title: string; source: string };
export type RenderedPage = Page & { html: string };

export function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\p{L}\p{N} _-]/gu, "")
    .trim()
    .replaceAll(" ", "-");
}

// Gives each heading of one page a unique id, in document order.
function idMaker() {
  const used = new Map<string, number>();
  return (text: string): string => {
    const base = slugify(text);
    const count = used.get(base) ?? 0;
    used.set(base, count + 1);
    return count === 0 ? base : `${base}-${count}`;
  };
}

function plain(tokens: Token[]): string {
  return tokens
    .map((t) => ("tokens" in t && t.tokens ? plain(t.tokens) : "text" in t ? t.text : ""))
    .join("");
}

function headings(tokens: Token[]): string[] {
  const make = idMaker();
  const ids: string[] = [];
  new Marked().walkTokens(tokens, (t) => {
    if (t.type === "heading") ids.push(make(plain((t as Tokens.Heading).tokens)));
  });
  return ids;
}

function markdownFiles(dir: string): string[] {
  const found: string[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!excluded.has(entry.name)) found.push(...markdownFiles(join(dir, entry.name)));
    } else if (entry.name.endsWith(".md")) {
      found.push(join(dir, entry.name));
    }
  }
  return found;
}

export function loadPages(docsDir = join(process.cwd(), "..", "docs")): Page[] {
  return markdownFiles(docsDir)
    .map((path) => relative(docsDir, path).split(sep).join("/"))
    .sort()
    .map((file) => {
      const source = readFileSync(join(docsDir, file), "utf8");
      const first = new Marked()
        .lexer(source)
        .find((t): t is Tokens.Heading => t.type === "heading" && t.depth === 1);
      return {
        slug: file.replace(/\.md$/, ""),
        file,
        title: first ? plain(first.tokens) : file.replace(/\.md$/, ""),
        source,
      };
    });
}

export function renderPages(pages: Page[]): RenderedPage[] {
  const lexed = new Map(pages.map((p) => [p.file, new Marked().lexer(p.source)]));
  const anchors = new Map(pages.map((p) => [p.file, new Set(headings(lexed.get(p.file) ?? []))]));
  const errors: string[] = [];

  const rendered = pages.map((page) => {
    const makeId = idMaker();
    const marked = new Marked({
      renderer: {
        heading({ tokens, depth }) {
          const id = makeId(plain(tokens));
          const text = this.parser.parseInline(tokens);
          return `<h${depth} id="${id}">${text} <a class="anchor" href="#${id}" aria-label="Link to this heading">#</a></h${depth}>\n`;
        },
        code({ text, lang }) {
          const language = (lang ?? "").split(/\s/)[0];
          const body =
            language === "horsegirl" || language === "horse"
              ? highlightHorsegirl(text)
              : escapeHtml(text);
          return `<pre><code${language ? ` class="language-${escapeHtml(language)}"` : ""}>${body}\n</code></pre>\n`;
        },
        link({ href, title, tokens }) {
          const target = resolveLink(page, href);
          const attr = title ? ` title="${escapeHtml(title)}"` : "";
          return `<a href="${escapeHtml(target)}"${attr}>${this.parser.parseInline(tokens)}</a>`;
        },
      },
    });
    return { ...page, html: marked.parser(lexed.get(page.file) ?? []) };
  });

  function resolveLink(page: Page, href: string): string {
    if (/^[a-z][a-z0-9+.-]*:/i.test(href) || href.startsWith("//")) return href;
    const [path = "", anchor] = href.split("#", 2);
    let file = page.file;
    if (path !== "") {
      file = posix.normalize(posix.join(dirname(page.file), path));
      if (!anchors.has(file)) {
        errors.push(
          `${page.file}: link to "${href}" names no page. Pages: docs/*.md except docs/concept.`,
        );
        return href;
      }
    }
    if (anchor !== undefined && !anchors.get(file)?.has(anchor)) {
      errors.push(`${page.file}: link to "${href}" names no heading "${anchor}" in ${file}.`);
    }
    const slug = file.replace(/\.md$/, "");
    return `${basePath}/docs/${slug}/${anchor === undefined ? "" : `#${anchor}`}`;
  }

  if (errors.length > 0) throw new Error(`Broken links in docs:\n  ${errors.join("\n  ")}`);
  return rendered;
}
