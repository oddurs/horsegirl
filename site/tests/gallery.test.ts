import assert from "node:assert/strict";
import { chmodSync, mkdtempSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { binaryPath, draw, drawGallery, root, sourceFiles } from "../lib/gallery.ts";

// A stand-in for the horsegirl binary: it prints the arguments it got.
function stub(body: string): string {
  const path = join(mkdtempSync(join(tmpdir(), "gallery-")), "horsegirl");
  writeFileSync(path, `#!/bin/sh\n${body}\n`);
  chmodSync(path, 0o755);
  return path;
}

test("the gallery holds at least six files of the corpus", () => {
  const names = sourceFiles().map((file) => file.slice(file.lastIndexOf("/") + 1));
  assert.ok(names.length >= 6, `found ${names.length}`);
  assert.ok(names.every((name) => name.endsWith(".horse")));
  assert.ok(names.includes("checkout.horse"));
});

test("a picture is what the binary prints for its file", () => {
  const file = sourceFiles()[0] as string;
  const picture = draw(stub('echo "<svg>$*</svg>"'), file);
  assert.equal(picture.svg, `<svg>draw -o - ${file}</svg>\n`);
  assert.equal(picture.command, `horsegirl draw ${picture.name}`);
  assert.ok(picture.source.length > 0);
});

test("a file that does not draw fails the build", () => {
  assert.throws(() => drawGallery(stub("echo E0001 >&2; exit 1"), sourceFiles().slice(0, 1)));
});

test("a missing binary names the command that builds it", () => {
  process.env.HORSEGIRL_BIN = "/nonexistent/horsegirl";
  try {
    assert.throws(() => binaryPath(), /cargo build -p horsegirl/);
  } finally {
    delete process.env.HORSEGIRL_BIN;
  }
});

test("no picture is committed under site/", () => {
  const found = readdirSync(join(root, "site"), { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && /\.(svg|png)$/.test(entry.name))
    .map((entry) => join(entry.parentPath, entry.name))
    .filter((path) => !/\/(node_modules|\.next|out)\//.test(path));
  assert.deepEqual(found, []);
});
