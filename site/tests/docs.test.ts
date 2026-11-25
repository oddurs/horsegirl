import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { loadPages, renderPages } from "../lib/docs.ts";
import { highlightHorsegirl } from "../lib/highlight.ts";

function tree(files: Record<string, string>): string {
  const dir = mkdtempSync(join(tmpdir(), "docs-"));
  for (const [name, text] of Object.entries(files)) {
    mkdirSync(join(dir, name, ".."), { recursive: true });
    writeFileSync(join(dir, name), text);
  }
  return dir;
}

test("a new Markdown file becomes a page, and docs/concept is skipped", () => {
  const dir = tree({
    "a.md": "# Alpha\n",
    "guide/b.md": "# Beta\n",
    "concept/c.md": "# Prototype\n",
  });
  assert.deepEqual(
    loadPages(dir).map((p) => [p.slug, p.title]),
    [
      ["a", "Alpha"],
      ["guide/b", "Beta"],
    ],
  );
  writeFileSync(join(dir, "d.md"), "# Delta\n");
  assert.deepEqual(
    loadPages(dir).map((p) => p.slug),
    ["a", "d", "guide/b"],
  );
});

test("headings get anchors, and a duplicate heading gets a suffix", () => {
  const [page] = renderPages(loadPages(tree({ "a.md": "# Same\n\n## Same\n" })));
  assert.match(page?.html ?? "", /<h1 id="same">/);
  assert.match(page?.html ?? "", /<h2 id="same-1">/);
});

test("links between pages resolve under the base path", () => {
  const pages = renderPages(
    loadPages(
      tree({ "a.md": "# A\n\n[b](guide/b.md#beta)\n", "guide/b.md": "# Beta\n[a](../a.md)\n" }),
    ),
  );
  assert.match(pages[0]?.html ?? "", /href="\/horsegirl\/docs\/guide\/b\/#beta"/);
  assert.match(pages[1]?.html ?? "", /href="\/horsegirl\/docs\/a\/"/);
});

test("a link to a missing page fails", () => {
  const pages = loadPages(tree({ "a.md": "# A\n\n[x](missing.md)\n" }));
  assert.throws(() => renderPages(pages), /a\.md: link to "missing\.md" names no page/);
});

test("a link to a missing heading fails", () => {
  const pages = loadPages(tree({ "a.md": "# A\n\n[x](#nowhere)\n" }));
  assert.throws(() => renderPages(pages), /names no heading "nowhere"/);
});

test("horsegirl code is highlighted and escaped", () => {
  const html = highlightHorsegirl("A -> B: <go>\nA -x C\nshow flow");
  assert.match(html, /<span class="hg-arrow">-&gt;<\/span>/);
  assert.match(html, /<span class="hg-arrow">-x<\/span>/);
  assert.match(html, /<span class="hg-keyword">show<\/span>/);
  assert.ok(html.includes("&lt;go&gt;"));
});
