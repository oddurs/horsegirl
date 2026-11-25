// The gallery draws each picture while the site builds. It runs the horsegirl
// binary of the same commit on a .horse file of the repository, so a picture
// shows what the program does today and no file of a picture is committed.
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

export type Picture = {
  name: string;
  command: string;
  source: string;
  svg: string;
};

// The build runs in site/, and the sources live at the repository root.
export const root = join(process.cwd(), "..");

// A fixed, named list: the reference file, then eight files of the converted
// corpus. The whole corpus is not drawn, because a build must stay fast.
export const sourceFilesList = [
  join("examples", "checkout.horse"),
  ...["0001", "0002", "0003", "0004", "0005", "0006", "0007", "0008"].map((id) =>
    join("corpus", "horse", `${id}.horse`),
  ),
];

export function sourceFiles(repo = root): string[] {
  return sourceFilesList.map((file) => join(repo, file));
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
