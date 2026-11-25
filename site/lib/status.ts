// What the binary does today, read from its own source. The landing page lists
// these commands, so the page cannot claim a command the binary lacks, and a new
// command reaches the page with no edit to it.
import { readFileSync } from "node:fs";
import { join } from "node:path";

export type Command = { name: string; does: string };

// The build runs in site/, and the crates live at the repository root.
const root = join(process.cwd(), "..");

export function readSpec(): string {
  return readFileSync(join(root, "crates", "horsegirl-core", "src", "spec.txt"), "utf8");
}

// Reads the "Commands:" block of the USAGE text in the binary's main.rs.
export function parseCommands(main: string): Command[] {
  const usage = main.match(/const USAGE: &str = "\\\n([\s\S]*?)";/)?.[1];
  if (usage === undefined) throw new Error("main.rs: no USAGE text");
  const block = usage.split("Commands:\n")[1];
  if (block === undefined) throw new Error("main.rs: USAGE has no Commands: block");
  return block
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map((line) => {
      const found = line.match(/^ {2}(\S+) +(.+)$/);
      if (!found?.[1] || !found[2]) throw new Error(`main.rs: cannot read command line "${line}"`);
      return { name: found[1], does: found[2] };
    });
}

export function readCommands(): Command[] {
  return parseCommands(readFileSync(join(root, "crates", "horsegirl", "src", "main.rs"), "utf8"));
}
