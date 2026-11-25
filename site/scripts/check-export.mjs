// Requests every page and asset the exported pages link to, under the base path.
// A link that escapes the base path, or names a file the export does not
// contain, fails the build.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { base, crawl, serve } from "./serve.mjs";

const server = await serve();
const { seen, failures } = await crawl(server.origin);
await server.close();

if (failures.length > 0) {
  console.error(`check-export: ${failures.length} broken link(s) under ${base}/:`);
  for (const failure of failures) console.error(`  ${failure}`);
  process.exit(1);
}
// The pages set their type in fonts the export carries. A font file, a style
// sheet or a page that names another origin would make the build depend on it.
const out = new URL("../out/", import.meta.url).pathname;
function walk(dir) {
  return readdirSync(dir, { withFileTypes: true, recursive: true })
    .filter((entry) => entry.isFile())
    .map((entry) => join(entry.parentPath, entry.name));
}
const files = walk(out);
const fonts = files.filter((file) => file.endsWith(".woff2"));
const licences = files.filter((file) => /licenses\/.*-OFL\.txt$/.test(file));
const foreign = files
  .filter((file) => /\.(css|html)$/.test(file))
  .filter((file) =>
    /fonts\.(googleapis|gstatic)\.com|@import\s+url\(\s*["']?https?:/.test(
      readFileSync(file, "utf8"),
    ),
  );
if (fonts.length !== 3 || licences.length !== 3 || foreign.length > 0) {
  console.error(
    `check-export: expected 3 font files and 3 licences and no foreign font origin; found ${fonts.length}, ${licences.length}, ${foreign.length}`,
  );
  process.exit(1);
}

// The gallery draws its pictures while the site builds, from the sources in
// the repository. A page with fewer than six pictures means a build lost them.
const gallery = readFileSync(join(out, "gallery", "index.html"), "utf8");
const pictures = gallery.match(/<svg[^>]*class="horsegirl"/g)?.length ?? 0;
if (pictures < 6) {
  console.error(`check-export: the gallery holds ${pictures} drawn pictures, expected at least 6`);
  process.exit(1);
}

console.log(`check-export: ${seen.size} paths served from ${base}/, all present`);
