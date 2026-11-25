// Finds and starts headless Chrome for the checks that load the exported site.
// check-a11y and check-lighthouse share it.
import { spawn } from "node:child_process";
import { accessSync, constants, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";

// accessSync throws when the file is missing or not executable; both mean "not here".
function executable(path) {
  try {
    accessSync(path, constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function findChrome() {
  if (process.env.CHROME) return process.env.CHROME;
  const names = ["google-chrome-stable", "google-chrome", "chromium", "chromium-browser"];
  for (const dir of (process.env.PATH ?? "").split(delimiter)) {
    const found = names.map((name) => join(dir, name)).find(executable);
    if (found) return found;
  }
  return null;
}

// Returns the path to Chrome. Without Chrome the check exits: with an error in
// CI, and with a skip note elsewhere. scripts/task makes the same choice for
// every optional tool.
export function requireChrome(check) {
  const chrome = findChrome();
  if (chrome) return chrome;
  if (process.env.CI) {
    console.error(`${check}: Chrome is required in CI; set CHROME to its path`);
    process.exit(1);
  }
  console.error(`${check}: Chrome not found; skipped (CI runs this check)`);
  process.exit(0);
}

// Starts Chrome with a fresh profile. `ready` gives the DevTools port.
export function launch(chrome, prefix) {
  const profile = mkdtempSync(join(tmpdir(), `${prefix}-`));
  // --no-sandbox: the runner's AppArmor policy blocks Chrome's sandbox, and the
  // browser loads only pages this build wrote.
  const proc = spawn(chrome, [
    "--headless=new",
    "--no-sandbox",
    "--disable-gpu",
    "--no-first-run",
    "--remote-debugging-port=0",
    `--user-data-dir=${profile}`,
    "about:blank",
  ]);
  const ready = new Promise((done, fail) => {
    let log = "";
    proc.stderr.on("data", (chunk) => {
      log += chunk;
      const found = log.match(/DevTools listening on (ws:\/\/\S+)/);
      if (found) done(Number(new URL(found[1]).port));
    });
    proc.on("exit", (code) => fail(new Error(`Chrome exited with ${code}:\n${log}`)));
  });
  const exited = new Promise((done) => proc.once("exit", done));
  // Chrome writes to its profile until it exits, so the profile goes after.
  // Its helper processes can still write for a moment after the main process
  // exits, and a file that appears during the delete gives ENOTEMPTY, so the
  // delete retries.
  const stop = async () => {
    if (proc.exitCode === null && proc.signalCode === null) {
      proc.kill();
      await exited;
    }
    rmSync(profile, { recursive: true, force: true, maxRetries: 10, retryDelay: 100 });
  };
  return { ready, stop };
}
