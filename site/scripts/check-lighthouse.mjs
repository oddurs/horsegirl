// Runs Lighthouse on the landing page, as a phone and as a desktop, and fails
// when performance or accessibility scores under the minimum. The page is the
// first one a visitor sees, so it carries the budget. The rule, and why
// performance is held only on CI, is in lib/lighthouse.ts.
import lighthouse from "lighthouse";
import desktop from "lighthouse/core/config/desktop-config.js";
import { judge, median, minimum, onCi } from "../lib/lighthouse.ts";
import { launch, requireChrome } from "./chrome.mjs";
import { base, serve } from "./serve.mjs";

const runs = 3;
const categories = ["performance", "accessibility"];
const forms = [
  { name: "mobile", config: undefined },
  { name: "desktop", config: desktop },
];
const ci = onCi(process.env);

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
      const scores = results.map(score);
      const middle = median(scores);
      const lhr = results[scores.indexOf(middle)];
      const line = `check-lighthouse: ${base}/ ${form.name} ${id} ${middle} (runs: ${scores.join(", ")})`;
      const verdict = judge(id, middle, ci);
      if (verdict === "pass") {
        console.log(line);
        continue;
      }
      if (verdict === "fail") {
        failed++;
        console.error(`${line}, under ${minimum}`);
      } else {
        console.error(`${line}, under ${minimum}; a note only, CI holds the threshold`);
      }
      // Only the audits that count toward this score name the cause.
      for (const ref of lhr.categories[id]?.auditRefs ?? []) {
        const audit = lhr.audits[ref.id];
        if (ref.weight > 0 && audit && audit.score !== null && audit.score < 0.9)
          console.error(`    ${audit.id}: ${audit.displayValue ?? audit.score}`);
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
console.log(`check-lighthouse: every held score is ${minimum} or more`);
