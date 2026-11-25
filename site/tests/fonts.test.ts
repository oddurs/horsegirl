import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { test } from "node:test";

const tokens = readFileSync(new URL("../app/tokens.css", import.meta.url), "utf8");
const fonts = readFileSync(new URL("../app/fonts.css", import.meta.url), "utf8");

// The first family of each stack the concept page names.
const families = ["--display", "--body", "--mono"].map((name) => {
  const match = tokens.match(new RegExp(`${name}:\\s*"([^"]+)"`));
  assert.ok(match, `${name} names no first family`);
  return match[1] as string;
});

test("each token font family has a local @font-face with a file on disk", () => {
  for (const family of families) {
    const face = fonts.match(
      new RegExp(`@font-face\\s*{[^}]*font-family:\\s*"${family}"[^}]*url\\("([^"]+)"\\)`),
    );
    assert.ok(face, `no @font-face for ${family}`);
    assert.ok(existsSync(new URL(`../app/${face[1]}`, import.meta.url)), `${face[1]} is missing`);
  }
});

test("a font face never loads from another origin", () => {
  assert.doesNotMatch(fonts, /url\(\s*["']?(https?:)?\/\//);
});

test("each family ships its licence", () => {
  for (const name of ["bricolage-grotesque", "instrument-sans", "jetbrains-mono"]) {
    assert.ok(existsSync(new URL(`../public/licenses/${name}-OFL.txt`, import.meta.url)), name);
  }
});
