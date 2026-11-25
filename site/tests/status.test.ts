import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";
import { parseCommands, readCommands } from "../lib/status.ts";

const main = `const USAGE: &str = "\\
horsegirl: an agent-first diagram language

Usage: horsegirl <command>

Commands:
  spec       Print the one-page language card
  --help     Print this help";
`;

test("the commands come from the Commands block of USAGE", () => {
  assert.deepEqual(parseCommands(main), [
    { name: "spec", does: "Print the one-page language card" },
    { name: "--help", does: "Print this help" },
  ]);
});

test("a USAGE text with no Commands block fails", () => {
  assert.throws(() => parseCommands('const USAGE: &str = "\\\nUsage: x";'), /no Commands: block/);
});

test("a main.rs with no USAGE text fails", () => {
  assert.throws(() => parseCommands("fn main() {}"), /no USAGE text/);
});

// The page lists what readCommands finds in main.rs. This test holds that list
// to what the built binary prints for --help.
test("the commands read from main.rs are the commands the binary prints", () => {
  const help = execFileSync(
    "cargo",
    ["run", "--quiet", "--locked", "-p", "horsegirl", "--", "--help"],
    {
      cwd: "..",
      encoding: "utf8",
    },
  );
  const printed = (help.split("Commands:\n")[1] ?? "")
    .trimEnd()
    .split("\n")
    .map((line) => line.trim().split(/ +/)[0]);
  assert.deepEqual(
    readCommands().map((command) => command.name),
    printed,
  );
});
