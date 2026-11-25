// Loads every exported page in headless Chrome, in light and dark mode, at a
// desktop and a phone width. On each it runs axe and fails on a serious or a
// critical finding. Then it presses Tab through the page and fails on any
// element that takes focus without a visible outline, and it fails when the
// page is wider than the window.
//
// It drives Chrome over the DevTools protocol with Node's own WebSocket, so the
// only dependency is axe itself.
import { createRequire } from "node:module";
import { launch, requireChrome } from "./chrome.mjs";
import { crawl, serve } from "./serve.mjs";

const axeSource = createRequire(import.meta.url)("axe-core").source;
const schemes = ["light", "dark"];
const widths = [
  { width: 1280, height: 900, mobile: false },
  { width: 390, height: 844, mobile: true },
];
const failing = new Set(["serious", "critical"]);
// axe rates these moderate, but a nested layout that adds a second <main> breaks
// every docs page at once, so they fail too.
const failingRules = new Set(["landmark-main-is-top-level", "landmark-no-duplicate-main"]);

const chrome = requireChrome("check-a11y");

function connect(url) {
  const socket = new WebSocket(url);
  const pending = new Map();
  const listeners = new Map();
  let next = 0;
  socket.addEventListener("message", ({ data }) => {
    const message = JSON.parse(data);
    if (message.id !== undefined) {
      const { done, fail } = pending.get(message.id);
      pending.delete(message.id);
      if (message.error) fail(new Error(message.error.message));
      else done(message.result);
    } else {
      listeners.get(message.method)?.(message.params);
    }
  });
  const send = (method, params = {}) =>
    new Promise((done, fail) => {
      const id = next++;
      pending.set(id, { done, fail });
      socket.send(JSON.stringify({ id, method, params }));
    });
  const once = (method, ms) =>
    new Promise((done, fail) => {
      const timer = setTimeout(() => fail(new Error(`no ${method} in ${ms} ms`)), ms);
      listeners.set(method, (params) => {
        clearTimeout(timer);
        listeners.delete(method);
        done(params);
      });
    });
  const open = new Promise((done, fail) => {
    socket.addEventListener("open", done);
    socket.addEventListener("error", () => fail(new Error(`cannot connect to ${url}`)));
  });
  return { open, send, once, close: () => socket.close() };
}

async function evaluate(cdp, expression) {
  const { result, exceptionDetails } = await cdp.send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (exceptionDetails) throw new Error(exceptionDetails.exception?.description ?? expression);
  return result.value;
}

const runAxe = `axe.run(document, { resultTypes: ["violations"] }).then((r) =>
  r.violations.map((v) => ({ id: v.id, impact: v.impact, help: v.help,
    nodes: v.nodes.map((n) => n.target.join(" ")) })))`;

// What has focus now, whether it shows a ring a person can see, and whether the
// walk reached it before. The walk ends on a repeat element, not a repeat name:
// two links with the same text are two stops.
const inspectFocus = `(() => {
  const el = document.activeElement;
  if (!el || el === document.body) return null;
  window.__checkA11ySeen ??= new WeakSet();
  const seen = window.__checkA11ySeen.has(el);
  window.__checkA11ySeen.add(el);
  const s = getComputedStyle(el);
  const name = el.tagName.toLowerCase() + (el.id ? "#" + el.id : "") +
    " " + JSON.stringify((el.getAttribute("aria-label") || el.textContent || "").trim().slice(0, 40));
  const visible = s.outlineStyle !== "none" && parseFloat(s.outlineWidth) >= 2 &&
    s.outlineColor !== "transparent" && !s.outlineColor.endsWith(", 0)");
  return { name, visible, seen };
})()`;

// How many elements Tab should reach: each one with a non-negative tabIndex that
// is enabled and rendered. The walk must stop at every one of them.
const countTabbable = `[...document.querySelectorAll("*")].filter((el) =>
  el.tabIndex >= 0 && !el.disabled && !el.inert && el.getClientRects().length > 0 &&
  el.checkVisibility({ visibilityProperty: true })).length`;

// The global reset in tokens.css takes the margins off headings and paragraphs,
// so prose that sets no spacing of its own runs together. In a docs page, two
// text blocks that follow each other must have a gap, and an h2 must use the
// full width of its column.
const checkProse = `(() => {
  const failures = [];
  const blocks = [...document.querySelectorAll(".docs :is(h1, h2, h3, h4, p)")];
  for (const el of blocks) {
    const next = el.nextElementSibling;
    if (!next || !next.matches("h1, h2, h3, h4, p")) continue;
    const gap = next.getBoundingClientRect().top - el.getBoundingClientRect().bottom;
    if (gap < 8) failures.push(el.tagName.toLowerCase() + " " +
      JSON.stringify(el.textContent.trim().slice(0, 40)) + " has a " + gap + "px gap to the next " +
      next.tagName.toLowerCase());
  }
  for (const h2 of document.querySelectorAll(".docs h2")) {
    if (getComputedStyle(h2).maxWidth !== "none")
      failures.push("h2 " + JSON.stringify(h2.textContent.trim().slice(0, 40)) +
        " has max-width " + getComputedStyle(h2).maxWidth);
  }
  return failures;
})()`;

// A page that is wider than the window scrolls sideways on a phone. A wide
// block must scroll inside its own box instead.
const checkWidth = `(() => {
  const { scrollWidth, clientWidth } = document.documentElement;
  return scrollWidth > clientWidth
    ? ["the page is " + scrollWidth + "px wide in a " + clientWidth + "px window"]
    : [];
})()`;

async function tab(cdp) {
  const key = { key: "Tab", code: "Tab", windowsVirtualKeyCode: 9 };
  await cdp.send("Input.dispatchKeyEvent", { type: "keyDown", ...key });
  await cdp.send("Input.dispatchKeyEvent", { type: "keyUp", ...key });
}

async function focusFailures(cdp) {
  const failures = [];
  const expected = await evaluate(cdp, countTabbable);
  let stops = 0;
  // A page has far fewer stops than this; the bound stops a focus trap.
  for (let i = 0; i < 200; i++) {
    await tab(cdp);
    const focus = await evaluate(cdp, inspectFocus);
    if (!focus || focus.seen) break;
    stops++;
    if (!focus.visible) failures.push(`no visible focus on ${focus.name}`);
  }
  if (stops === 0) failures.push("Tab reached no element");
  if (stops !== expected)
    failures.push(`Tab made ${stops} stops, the page has ${expected} tabbable elements`);
  return { failures, stops };
}

const server = await serve();
const browser = launch(chrome, "check-a11y");
let failed = 0;
let runs = 0;
try {
  const port = await browser.ready;
  const { pages, failures } = await crawl(server.origin);
  if (failures.length > 0) throw new Error(`pages failed to load: ${failures.join(", ")}`);
  const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
  const target = targets.find((t) => t.type === "page");
  const cdp = connect(target.webSocketDebuggerUrl);
  await cdp.open;
  await cdp.send("Page.enable");
  for (const path of pages) {
    for (const scheme of schemes) {
      for (const size of widths) {
        const where = `${path} ${scheme} ${size.width}px`;
        await cdp.send("Emulation.setEmulatedMedia", {
          features: [{ name: "prefers-color-scheme", value: scheme }],
        });
        await cdp.send("Emulation.setDeviceMetricsOverride", { ...size, deviceScaleFactor: 1 });
        const loaded = cdp.once("Page.loadEventFired", 30000);
        await cdp.send("Page.navigate", { url: server.origin + path });
        await loaded;
        await evaluate(cdp, axeSource);
        const violations = await evaluate(cdp, runAxe);
        const focus = await focusFailures(cdp);
        const prose = await evaluate(cdp, checkProse);
        const width = await evaluate(cdp, checkWidth);
        runs++;
        for (const v of violations) {
          const line = `${where}: ${v.impact} ${v.id}: ${v.help}\n    ${v.nodes.join("\n    ")}`;
          if (failing.has(v.impact) || failingRules.has(v.id)) {
            failed++;
            console.error(`check-a11y: ${line}`);
          } else {
            console.log(`check-a11y: note: ${line}`);
          }
        }
        for (const failure of [...focus.failures, ...prose, ...width]) {
          failed++;
          console.error(`check-a11y: ${where}: ${failure}`);
        }
        console.log(
          `check-a11y: ${where}: ${violations.length} finding(s), ${focus.stops} focus stops`,
        );
      }
    }
  }
  cdp.close();
} finally {
  await browser.stop();
  await server.close();
}

if (failed > 0) {
  console.error(`check-a11y: ${failed} failure(s) in ${runs} page loads`);
  process.exit(1);
}
console.log(`check-a11y: ${runs} page loads, no serious finding, focus visible at every stop`);
