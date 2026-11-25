import assert from "node:assert/strict";
import { test } from "node:test";
import { highlightHorsegirl } from "../lib/highlight.ts";

test("a link target before a colon is not a fact key", () => {
  const html = highlightHorsegirl("Web app -> API gateway: POST /checkout");
  assert.ok(!html.includes('<span class="hg-key">'));
  assert.ok(html.includes('<span class="hg-arrow">-&gt;</span>'));
});

test("a fact key inside braces is still a key", () => {
  const html = highlightHorsegirl("Payments { owner: Commerce }");
  assert.ok(html.includes('<span class="hg-key">owner</span>'));
});
