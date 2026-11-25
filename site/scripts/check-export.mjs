// Requests every page and asset the exported pages link to, under the base path.
// A link that escapes the base path, or names a file the export does not
// contain, fails the build.
import { base, crawl, serve } from "./serve.mjs";

const server = await serve();
const { seen, failures } = await crawl(server.origin);
await server.close();

if (failures.length > 0) {
  console.error(`check-export: ${failures.length} broken link(s) under ${base}/:`);
  for (const failure of failures) console.error(`  ${failure}`);
  process.exit(1);
}
console.log(`check-export: ${seen.size} paths served from ${base}/, all present`);
