import type { NextConfig } from "next";
import { basePath } from "./lib/base.ts";

const nextConfig: NextConfig = {
  // GitHub Pages serves files and runs no server, so every page is built ahead.
  output: "export",
  // The project site lives at https://oddurs.github.io/horsegirl/.
  basePath,
  // Pages emits /docs/index.html, which a static host serves at /docs/ with no rewrite.
  trailingSlash: true,
  // The repository's AGENTS.md carries the instructions. Without this, `next dev`
  // writes site/AGENTS.md for some callers and not others, so the tree depends on who ran it.
  agentRules: false,
};

export default nextConfig;
