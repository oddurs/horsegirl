// One-off probe for item BUG-158: the drawn width of each label, with
// textLength removed, against the width that text::measure gave it.
import { readFileSync } from "node:fs";
import { chromium, firefox, webkit } from "playwright";

const svg = readFileSync(new URL("./labels.svg", import.meta.url), "utf8");
const engines = { chromium, firefox, webkit };
const wanted = (process.env.ENGINES ?? "chromium,firefox,webkit").split(",");
let worst = 0;
for (const name of wanted) {
  const browser = await engines[name].launch();
  const page = await browser.newPage();
  await page.setContent(`<!doctype html><body style="margin:0">${svg}</body>`);
  const rows = await page.evaluate(() =>
    [...document.querySelectorAll("text.label")].map((t) => {
      const measured = Number(t.getAttribute("textLength"));
      t.removeAttribute("textLength");
      t.removeAttribute("lengthAdjust");
      return { text: JSON.stringify(t.textContent), measured, drawn: t.getComputedTextLength() };
    }),
  );
  for (const r of rows) {
    const diff = r.drawn - r.measured;
    if (r.text.includes("Wide")) worst = Math.max(worst, Math.abs(diff));
    console.log(`${name}\t${r.text}\tmeasured ${r.measured.toFixed(2)}\tdrawn ${r.drawn.toFixed(2)}\tdiff ${diff.toFixed(2)}`);
  }
  await browser.close();
}
console.log(`largest difference on the two labels: ${worst.toFixed(2)} px`);
process.exit(worst <= 1 ? 0 : 1);
