// Runs a wasm32-wasip1 program under the WASI support built into Node.
// Usage: node scripts/wasi-run.mjs <program.wasm> [arguments]
//
// Cargo calls this as the runner for the wasm32-wasip1 target, so
// `cargo test --target wasm32-wasip1` runs the core tests in WebAssembly with
// the Node that the site already needs, and no other runtime.
import { readFile } from "node:fs/promises";
import { argv, env, exit } from "node:process";
import { WASI } from "node:wasi";

const [program, ...args] = argv.slice(2);
if (!program) {
  console.error("usage: node scripts/wasi-run.mjs <program.wasm> [arguments]");
  exit(2);
}

// The tests read no file at run time, so the program gets no directory.
const wasi = new WASI({ version: "preview1", args: [program, ...args], env });
const module = await WebAssembly.compile(await readFile(program));
const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
exit(wasi.start(instance));
