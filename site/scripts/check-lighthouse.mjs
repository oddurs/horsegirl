// Runs Lighthouse on the landing page, as a phone and as a desktop, and fails
// when performance or accessibility scores under 95. The page is the first one a
// visitor sees, so it carries the budget.
import lighthouse from "lighthouse";
import desktop from "lighthouse/core/config/desktop-config.js";
import { launch, requireChrome } from "./chrome.mjs";
import { base, serve } from "./serve.mjs";

const minimum = 95;
const categories = ["performance", "accessibility"];
const forms = [
  { name: "mobile", config: undefined },
  { name: "desktop", config: desktop },
];

const chrome = requireChrome("check-lighthouse");
const server = await serve();
const browser = launch(chrome, "check-lighthouse");
let failed = 0;
try {
  const port = await browser.ready;
  const url = `${server.origin}${base}/`;
  for (const form of forms) {
    const result = await lighthouse(
      url,
      { port, onlyCategories: categories, logLevel: "error", output: "json" },
      form.config,
    );
    if (!result) throw new Error(`Lighthouse returned no result for ${url}`);
    if (result.lhr.runtimeError) throw new Error(result.lhr.runtimeError.message);
    for (const id of categories) {
      const score = Math.round((result.lhr.categories[id]?.score ?? 0) * 100);
      const line = `check-lighthouse: ${base}/ ${form.name} ${id} ${score}`;
      if (score < minimum) {
        failed++;
        console.error(`${line}, under ${minimum}`);
        // Only the audits that count toward this score name the cause.
        for (const ref of result.lhr.categories[id]?.auditRefs ?? []) {
          const audit = result.lhr.audits[ref.id];
          if (ref.weight > 0 && audit && audit.score !== null && audit.score < 0.9)
            console.error(`    ${audit.id}: ${audit.displayValue ?? audit.score}`);
        }
      } else {
        console.log(line);
      }
    }
  }
} finally {
  await browser.stop();
  await server.close();
}

if (failed > 0) {
  console.error(`check-lighthouse: ${failed} score(s) under ${minimum}`);
  process.exit(1);
}
console.log(`check-lighthouse: every score is ${minimum} or more`);
