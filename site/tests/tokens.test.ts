import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const concept = readFileSync(new URL("../../docs/concept/index.html", import.meta.url), "utf8");
const site = readFileSync(new URL("../app/tokens.css", import.meta.url), "utf8");

// The text between the first `{` after `from` and the brace that closes it.
function block(text: string, from: string): string {
  const at = text.indexOf(from);
  assert.notEqual(at, -1, `no ${JSON.stringify(from)} found`);
  const open = text.indexOf("{", at + from.length - 1);
  let depth = 0;
  for (let i = open; i < text.length; i++) {
    if (text[i] === "{") depth++;
    if (text[i] === "}" && --depth === 0) return text.slice(open + 1, i);
  }
  throw new Error(`${from} is not closed`);
}

// Custom properties, written the same way whatever the formatter did:
// lower case, no spaces, and `.5rem` as `0.5rem`.
function vars(css: string): Map<string, string> {
  const out = new Map<string, string>();
  for (const [, name, value] of css.matchAll(/--([\w-]+)\s*:\s*([^;]+?)\s*(?:;|$)/g)) {
    const same = (value as string)
      .replace(/\s+/g, "")
      .replace(/(^|[^\d])\.(\d)/g, "$10.$2")
      .toLowerCase();
    out.set(name as string, same);
  }
  return out;
}

// The site says `ground` where the concept page says `bg`.
const rename = (name: string) => (name === "bg" ? "ground" : name);

function assertMatches(want: Map<string, string>, got: Map<string, string>, where: string) {
  assert.ok(want.size > 0, `the concept page has no tokens in ${where}`);
  for (const [name, value] of want) {
    assert.equal(got.get(rename(name)), value, `--${rename(name)} in ${where}`);
  }
}

const v8 = concept.slice(concept.indexOf("/* ---- v8"));

test("the light page tokens match the concept page", () => {
  const want = vars(block(v8, ":root{"));
  for (const name of ["bg", "ink", "muted", "line", "accent", "r1", "r2", "r3", "s1", "s6"]) {
    assert.ok(want.has(name), `the concept page lost --${name}`);
  }
  assertMatches(want, vars(block(site, ":root {")), "the light tokens");
});

test("the dark page tokens match the concept page", () => {
  const want = vars(block(v8, ':root[data-theme="dark"]{'));
  const dark = block(site, "@media (prefers-color-scheme: dark)");
  assertMatches(want, vars(block(dark, ":root {")), "the dark tokens");
});

test("the diagram tokens match the concept page", () => {
  const themes = concept.slice(concept.indexOf("/* ---- diagram themes"));
  const want = vars(block(themes, ":root{"));
  assert.ok(want.has("d-ink") && want.has("d-paper"), "the concept page lost its diagram tokens");
  assertMatches(want, vars(block(site, ":root {")), "the diagram tokens");
});
