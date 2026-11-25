#!/usr/bin/env python3
"""Count the names in the Mermaid corpus that a bare horsegirl name cannot hold.

Run from anywhere: python3 corpus/measure.py
It reads corpus/flowcharts/*.mmd and prints the same report on every machine.
SPIKE-17 quotes this report.
"""

import collections
import html
import pathlib
import re
import sys

CORPUS = pathlib.Path(__file__).resolve().parent / "flowcharts"

# Shape openers and their closers, longest first so that "((" wins over "(".
SHAPES = [
    ("(((", (")))",)),
    ("([", ("])",)),
    ("[[", ("]]",)),
    ("[(", (")]",)),
    ("((", ("))",)),
    ("{{", ("}}",)),
    ("[/", ("/]", "\\]")),
    ("[\\", ("\\]", "/]")),
    ("(", (")",)),
    ("[", ("]",)),
    ("{", ("}",)),
    (">", ("]",)),
]
SKIP_WORDS = ("classDef", "class", "style", "linkStyle", "click", "direction", "end",
              "accTitle", "accDescr", "flowchart", "graph")
LINK = re.compile(r"<?(?:-{2,}|={2,}|-\.+-|~{3,})(?:>|[ox](?=[\s\w]|$))?")
# Text on a link: "-- text -->", "== text ==>", "-. text .->". The text may wrap once.
TEXT_LINK = re.compile(
    r"<?(?:(?:--|==)(?=[\s\"])|-\.(?![-.]))\s*([^\n]+?(?:\n[^\n]+?)?)\s*"
    r"(-{2,}>|-{3,}|-{2,}[ox]|={2,}>|={3,}|\.-+>|\.-+)"
)
ID_STOP = set(" \t\n;[](){}<>|&:\"'@")
BREAK = re.compile(r"<br\s*/?>|\\n|\n", re.I)
TAG = re.compile(r"</?[a-zA-Z][^>]*>")
ICON = re.compile(r"\bfa[bkrs]?:fa-[\w-]+")
KEYWORDS = ("show", "story")


def decode(text):
    """Mermaid writes entities as #name; or #35;. Turn both into characters."""
    text = re.sub(r"#(\w+);", lambda m: "&" + ("#" + m[1] if m[1].isdigit() else m[1]) + ";", text)
    return html.unescape(text)


def unquote(text):
    text = text.strip()
    if len(text) >= 2 and text[0] == '"' and text[-1] == '"':
        text = text[1:-1]
    if len(text) >= 2 and text[0] == "`" and text[-1] == "`":
        text = text[1:-1]
    return text


class Chart:
    def __init__(self, text):
        self.text = text
        self.pos = 0
        self.labels = {}  # node id -> label, in first-definition order
        self.order = []
        self.mentions = collections.Counter()
        self.groups = []
        self.edge_labels = []
        self.unparsed = 0

    def peek(self, s):
        return self.text.startswith(s, self.pos)

    def skip_space(self, newlines=False):
        chars = " \t\n" if newlines else " \t"
        while self.pos < len(self.text) and self.text[self.pos] in chars:
            self.pos += 1

    def rest_of_line(self):
        end = self.text.find("\n", self.pos)
        end = len(self.text) if end < 0 else end
        line = self.text[self.pos : end]
        self.pos = end
        return line

    def node_id(self):
        start = self.pos
        while self.pos < len(self.text) and self.text[self.pos] not in ID_STOP:
            if self.pos > start and (LINK.match(self.text, self.pos) or self.peek("-.")):
                break
            self.pos += 1
        return self.text[start : self.pos]

    def shape(self):
        for opener, closers in SHAPES:
            if not self.peek(opener):
                continue
            self.pos += len(opener)
            body_start = self.pos
            self.skip_space()
            if self.peek('"'):
                end = self.text.find('"', self.pos + 1)
                if end < 0:
                    return None
                self.pos = end + 1
                self.skip_space()
            ends = [(self.text.find(c, self.pos), c) for c in closers]
            ends = [(i, c) for i, c in ends if i >= 0]
            if not ends:
                return None
            i, c = min(ends)
            label = self.text[body_start:i]
            self.pos = i + len(c)
            return label
        return None

    def node(self):
        nid = self.node_id()
        if not nid:
            return None
        label = self.shape()
        if self.peek("@{"):
            end = self.text.find("}", self.pos)
            body = self.text[self.pos + 2 : end if end >= 0 else len(self.text)]
            m = re.search(r'label\s*:\s*"([^"]*)"', body)
            label = m[1] if m else label
            self.pos = len(self.text) if end < 0 else end + 1
        if self.peek(":::"):
            self.pos += 3
            self.node_id()
        self.mentions[nid] += 1
        if nid not in self.labels:
            self.labels[nid] = None
            self.order.append(nid)
        if label is not None and self.labels[nid] is None:
            self.labels[nid] = label
        return nid

    def nodes(self):
        if self.node() is None:
            return False
        while True:
            self.skip_space()
            if not self.peek("&"):
                return True
            self.pos += 1
            self.skip_space()
            if self.node() is None:
                return False

    def link(self):
        m = TEXT_LINK.match(self.text, self.pos)
        if m:
            self.edge_labels.append(m[1])
            self.pos = m.end()
            return True
        m = LINK.match(self.text, self.pos)
        if not m:
            return False
        self.pos = m.end()
        self.skip_space()
        if self.peek("|"):
            end = self.text.find("|", self.pos + 1)
            if end < 0:
                return False
            self.edge_labels.append(self.text[self.pos + 1 : end])
            self.pos = end + 1
        return True

    def statement(self):
        word = re.match(r"[A-Za-z_]\w*", self.text[self.pos :])
        word = word[0] if word else ""
        if word == "subgraph":
            self.pos += len(word)
            line = self.rest_of_line().strip()
            m = re.match(r"[^\s\[]+\s*\[(.*)\]\s*$", line)
            self.groups.append(m[1] if m else line)
            return
        if word == "accDescr" and re.match(r"accDescr\s*\{", self.text[self.pos :]):
            end = self.text.find("}", self.pos)
            self.pos = len(self.text) if end < 0 else end + 1
            return
        if word in SKIP_WORDS and not re.match(r"\w*[\[\(\{>-]", self.text[self.pos + len(word):]):
            self.rest_of_line()
            return
        start = self.pos
        if self.nodes():
            while True:
                # A link may start on the next line, so look past one newline.
                before = self.pos
                self.skip_space(newlines=True)
                if not self.link():
                    self.pos = before
                    break
                self.skip_space(newlines=True)
                if not self.nodes():
                    break
        self.skip_space()
        if self.pos < len(self.text) and self.text[self.pos] not in "\n;":
            self.unparsed += 1
            self.rest_of_line()
        elif self.pos == start:
            self.pos += 1

    def parse(self):
        lines = self.text.split("\n")
        body = []
        in_front = lines and lines[0].strip() == "---"
        header_done = False
        for i, line in enumerate(lines):
            if in_front:
                if i > 0 and line.strip() == "---":
                    in_front = False
                body.append("")
                continue
            if line.strip().startswith("%%"):
                body.append("")
                continue
            if not header_done and line.strip():
                header_done = True
                line = re.sub(r"^\s*(flowchart|graph)(\s+\w+)?\s*;?", "", line, flags=re.I)
            body.append(line)
        self.text = "\n".join(body)
        self.pos = 0
        while True:
            self.skip_space(newlines=True)
            while self.peek(";"):
                self.pos += 1
                self.skip_space(newlines=True)
            if self.pos >= len(self.text):
                return self
            self.statement()


def plain(raw, first_line=False):
    """The name a converter would write after its safe rewrites."""
    text = decode(unquote(raw))
    if first_line:
        text = next((part for part in BREAK.split(text) if TAG.sub("", part).strip()), "")
    text = BREAK.sub(" ", text)
    text = TAG.sub("", text)
    text = ICON.sub("", text)
    return " ".join(text.split())


def rest_of_label(raw):
    """The lines after the first: what a converter writes as a note fact."""
    parts = BREAK.split(decode(unquote(raw)))
    first = next((i for i, part in enumerate(parts) if TAG.sub("", part).strip()), len(parts))
    text = ICON.sub("", TAG.sub("", " ".join(parts[first + 1 :])))
    return " ".join(text.split())


def rewrites(raw):
    text = decode(unquote(raw))
    found = []
    if BREAK.search(text):
        found.append("line break")
    if TAG.sub("", BREAK.sub("", text)) != BREAK.sub("", text) or ICON.search(text):
        found.append("markup")
    return found


def causes(name):
    found = []
    if not name:
        return ["empty"]
    if "," in name:
        found.append("comma")
    if ":" in name:
        found.append("colon")
    if "{" in name or "}" in name:
        found.append("brace")
    if "->" in name:
        found.append("arrow ->")
    if re.search(r"-x(?![A-Za-z])", name):
        found.append("arrow -x")
    if name.endswith(" ?"):
        found.append("final ' ?'")
    if "@" in name:
        found.append("@")
    if '"' in name:
        found.append('double quote')
    if name.split()[0] in KEYWORDS:
        found.append("keyword")
    return found


def one_edit(a, b):
    """True when one insertion, deletion or substitution turns a into b."""
    if a == b or abs(len(a) - len(b)) > 1:
        return False
    if len(a) > len(b):
        a, b = b, a
    i = 0
    while i < len(a) and a[i] == b[i]:
        i += 1
    return a[i + (len(a) == len(b)) :] == b[i + 1 :]


def pct(n, d):
    return f"{100 * n / d:.1f}%" if d else "n/a"


def row(label, n, note=""):
    print(f"{label:44}{n:>6}  {note}".rstrip())


def main():
    files = sorted(CORPUS.glob("*.mmd"))
    if not files:
        sys.exit(f"measure: no flowcharts in {CORPUS}")
    names = []  # (file, kind, raw, plain, plain first line)
    edge_labels = []
    unparsed = 0
    unlabelled = 0
    shared = collections.Counter()  # a name that two or more nodes of one chart carry
    charts_with_shared = 0
    typo_names = 0
    charts_with_typo = 0
    for path in files:
        chart = Chart(path.read_text(encoding="utf-8")).parse()
        unparsed += chart.unparsed
        seen = collections.Counter()
        for nid in chart.order:
            raw = chart.labels[nid]
            if raw is None:
                unlabelled += 1
                raw = nid
            names.append((path.name, "node", raw, plain(raw), plain(raw, True)))
            seen[plain(raw)] += 1
        dups = {name: n for name, n in seen.items() if n > 1}
        shared.update(dups)
        charts_with_shared += bool(dups)
        # E0412 fires on a name used once that is one edit from another name.
        # Bare ids such as A and B are left out: a converter would not keep them.
        used = collections.Counter()
        for nid in chart.order:
            if chart.labels[nid] is not None:
                used[plain(chart.labels[nid])] += chart.mentions[nid]
        flagged = [a for a in used if used[a] == 1 and any(one_edit(a, b) for b in used)]
        typo_names += len(flagged)
        charts_with_typo += bool(flagged)
        for raw in chart.groups:
            names.append((path.name, "group", raw, plain(raw), plain(raw, True)))
        edge_labels.extend((path.name, raw, plain(raw)) for raw in chart.edge_labels)

    total = len(names)
    nodes = sum(1 for n in names if n[1] == "node")
    labelled = nodes - unlabelled
    quoted = [n for n in names if causes(n[3])]
    charts_quoted = len({n[0] for n in quoted})
    multi = sum(1 for n in quoted if "line break" in rewrites(n[2]))
    colon_tight = sum(
        1 for n in quoted if causes(n[3]) == ["colon"] and not re.search(r":(\s|$)", n[3])
    )
    first = sum(1 for n in names if causes(n[4]))
    # A fact value sits inside braces, so a comma, a colon or a brace in it is reserved too.
    saved = [n for n in quoted if not causes(n[4])]
    moved = sum(1 for n in saved if {"comma", "colon", "brace"} & set(causes(rest_of_label(n[2]))))
    rewritten = sum(1 for n in names if rewrites(n[2]))
    question = sum(1 for n in names if n[3].endswith("?") and not n[3].endswith(" ?"))
    by_cause = collections.Counter(c for n in names for c in causes(n[3]))
    by_rewrite = collections.Counter(c for n in names for c in rewrites(n[2]))
    shared_nodes = sum(shared.values())
    top_shared = sorted(shared.items(), key=lambda kv: (-kv[1], kv[0]))[:5]

    row("flowcharts", len(files))
    row("names", total, f"{nodes} nodes, {total - nodes} subgraph titles")
    row("nodes with no label (the id is the name)", unlabelled, pct(unlabelled, nodes) + " of nodes")
    row("lines not understood", unparsed)
    print()
    row("names that need quotes", len(quoted), pct(len(quoted), total))
    row("  flowcharts that hold one", charts_quoted, pct(charts_quoted, len(files)))
    row("  names that also hold a line break", multi)
    row("  names whose only colon has no space after", colon_tight)
    row("names whose first line needs quotes", first, pct(first, total))
    row("  names that line one alone saves", len(saved), pct(len(saved), len(quoted)) + " of names that need quotes")
    row("    other lines hold a comma, colon or brace", moved, pct(moved, len(saved)))
    row("names that need a safe rewrite", rewritten, pct(rewritten, total))
    row("names that end in '?' with no space", question, pct(question, total) + ", item 26 keeps them bare")
    row("labels that E0412 would call a typo", typo_names,
        f"{pct(typo_names, labelled)} of labelled nodes, in {charts_with_typo} flowcharts")
    row("nodes that share a name in one flowchart", shared_nodes,
        f"{pct(shared_nodes, nodes)} of nodes, in {charts_with_shared} flowcharts")
    row("  most common", len(shared), " | ".join(f"{name!r} x{n}" for name, n in top_shared))
    print()
    print("causes that need quotes (a name can have several)")
    for cause, n in sorted(by_cause.items(), key=lambda kv: (-kv[1], kv[0])):
        examples = sorted({x[3] for x in names if cause in causes(x[3])}, key=lambda t: (len(t), t))
        shown = " | ".join(repr(t) for t in examples[:3])
        row(f"  {cause}", n, f"{pct(n, total)}  shortest: {shown}")
    print()
    print("causes that a converter rewrites without quotes")
    for cause, n in sorted(by_rewrite.items(), key=lambda kv: (-kv[1], kv[0])):
        row(f"  {cause}", n, pct(n, total))
    print()
    row("link labels", len(edge_labels))
    for cause in ("final ' ?'", "@"):
        n = sum(1 for e in edge_labels if e[2] and cause in causes(e[2]))
        row(f"  {cause}", n, pct(n, len(edge_labels)))


if __name__ == "__main__":
    main()
