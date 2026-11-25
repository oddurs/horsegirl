// Serves the static export under the base path, as GitHub Pages does, and walks
// every page and asset the pages link to. check-export and check-a11y share it.
import { readFile, stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, join, normalize } from "node:path";

export const base = "/horsegirl";
const out = new URL("../out/", import.meta.url).pathname;
const types = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".txt": "text/plain",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".ico": "image/x-icon",
  ".woff2": "font/woff2",
};

async function resolve(pathname) {
  if (!pathname.startsWith(`${base}/`)) return null;
  const rel = normalize(decodeURIComponent(pathname.slice(base.length)));
  if (rel.includes("..")) return null;
  let file = join(out, rel);
  if (rel.endsWith("/")) file = join(file, "index.html");
  try {
    return (await stat(file)).isFile() ? file : null;
  } catch {
    return null;
  }
}

// Starts the server on a free port. Returns its origin and a function that stops it.
export async function serve() {
  const server = createServer(async (req, res) => {
    const file = await resolve(new URL(req.url, "http://localhost").pathname);
    if (!file) {
      res.writeHead(404).end();
      return;
    }
    res.writeHead(200, { "content-type": types[extname(file)] ?? "application/octet-stream" });
    res.end(await readFile(file));
  });
  await new Promise((done) => server.listen(0, "127.0.0.1", done));
  return {
    origin: `http://127.0.0.1:${server.address().port}`,
    close: () => new Promise((done) => server.close(done)),
  };
}

// Requests every path reachable from the base path. Returns the paths it saw,
// the HTML pages among them in the order found, and each path that failed.
export async function crawl(origin) {
  const failures = [];
  const pages = [];
  const seen = new Set([`${base}/`]);
  const queue = [`${base}/`];
  while (queue.length > 0) {
    const path = queue.shift();
    const res = await fetch(origin + path);
    if (res.status !== 200) {
      failures.push(`${path}: ${res.status}`);
      continue;
    }
    if (!(res.headers.get("content-type") ?? "").startsWith("text/html")) continue;
    pages.push(path);
    // Next writes <link rel="preconnect" href="/"> once a style sheet names a font
    // file. It points at the host root, not at a page of this site.
    const html = (await res.text()).replace(/<link rel="preconnect"[^>]*>/g, "");
    for (const [, url] of html.matchAll(/\s(?:href|src)="([^"]+)"/g)) {
      const target = new URL(url, origin + path);
      if (target.origin !== origin || seen.has(target.pathname)) continue;
      seen.add(target.pathname);
      queue.push(target.pathname);
    }
  }
  return { seen, pages, failures };
}
