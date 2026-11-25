// A small highlighter for horsegirl source. The language is line based, so a
// per-line scan is enough. It emits spans with `hg-` classes and escapes all text.

export function escapeHtml(text: string): string {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

const token =
  /(?<evidence>@ \S+)|(?<arrow>-x|->)|(?<keyword>\b(?:story|show|of|from|by)\b)|(?<key>[A-Za-z][\w ]*(?=:\s))|(?<punct>[{}?,:])/g;

function highlightLine(line: string): string {
  let out = "";
  let last = 0;
  for (const match of line.matchAll(token)) {
    const [text] = match;
    const kind = Object.entries(match.groups ?? {}).find(([, value]) => value !== undefined)?.[0];
    out += escapeHtml(line.slice(last, match.index));
    out += `<span class="hg-${kind}">${escapeHtml(text)}</span>`;
    last = match.index + text.length;
  }
  return out + escapeHtml(line.slice(last));
}

export function highlightHorsegirl(source: string): string {
  return source.split("\n").map(highlightLine).join("\n");
}
