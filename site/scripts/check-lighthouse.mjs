// Runs Lighthouse on the landing page, as a phone and as a desktop, and fails
// when performance or accessibility scores under 95. The page is the first one a
// visitor sees, so it carries the budget.
//
// The performance score moves by a few points with the load on the machine, so
// each score is the median of three runs, as the Lighthouse docs advise.
import lighthouse from "lighthouse";
import desktop from "lighthouse/core/config/desktop-config.js";
import { launch, requireChrome } from "./chrome.mjs";
import { base, serve } from "./serve.mjs";

const minimum = 95;
const runs = 3;
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
    const results = [];
    for (let i = 0; i < runs; i++) {
      const result = await lighthouse(
        url,
        { port, onlyCategories: categories, logLevel: "error", output: "json" },
        form.config,
      );
      if (!result) throw new Error(`Lighthouse returned no result for ${url}`);
      if (result.lhr.runtimeError) throw new Error(result.lhr.runtimeError.message);
      results.push(result.lhr);
    }
    for (const id of categories) {
      const score = (lhr) => Math.round((lhr.categories[id]?.score ?? 0) * 100);
      const sorted = results.toSorted((a, b) => score(a) - score(b));
      const median = sorted[Math.floor(runs / 2)];
      const line = `check-lighthouse: ${base}/ ${form.name} ${id} ${score(median)} (runs: ${results.map(score).join(", ")})`;
      if (score(median) < minimum) {
        failed++;
        console.error(`${line}, under ${minimum}`);
        // Only the audits that count toward this score name the cause.
        for (const ref of median.categories[id]?.auditRefs ?? []) {
          const audit = median.audits[ref.id];
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
