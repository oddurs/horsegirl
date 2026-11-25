// The gallery draws each picture while the site builds. It runs the horsegirl
// binary of the same commit on a .horse file of the repository, so a picture
// shows what the program does today and no file of a picture is committed.
import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

export type Picture = {
  name: string;
  command: string;
  source: string;
  svg: string;
};

// The build runs in site/, and the sources live at the repository root.
export const root = join(process.cwd(), "..");

// The reference file, then the diagrams of the layered-layout spike. These are
// the same files that the layout tests draw.
export const sourceDirs = ["examples", join("spikes", "layered-layout", "corpus")];

export function sourceFiles(repo = root): string[] {
  return sourceDirs.flatMap((dir) =>
    readdirSync(join(repo, dir))
      .filter((name) => name.endsWith(".horse"))
      .sort()
      .map((name) => join(repo, dir, name)),
  );
}

export function binaryPath(repo = root): string {
  const path = process.env.HORSEGIRL_BIN ?? join(repo, "target", "debug", "horsegirl");
  if (!existsSync(path)) {
    throw new Error(
      `gallery: no horsegirl binary at ${path}. Run "cargo build -p horsegirl", or set HORSEGIRL_BIN.`,
    );
  }
  return path;
}

// A file that does not draw fails the build: a gallery must not hide an error.
export function draw(binary: string, file: string): Picture {
  const name = file.slice(file.lastIndexOf("/") + 1);
  const svg = execFileSync(binary, ["draw", "-o", "-", file], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  return { name, command: `horsegirl draw ${name}`, source: readFileSync(file, "utf8"), svg };
}

export function drawGallery(binary = binaryPath(), files = sourceFiles()): Picture[] {
  return files.map((file) => draw(binary, file));
}
