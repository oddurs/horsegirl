#!/usr/bin/env python3
"""Convert the Mermaid corpus to the subset of horsegirl that this spike reads.

Run from this directory:

    python3 -I mermaid.py ../../corpus/flowcharts target/public

It writes one NNNN.horse file for each flowchart that converts, and prints one line for
each flowchart that does not, with the reason. The output is the same on every machine.
Item 0125 uses it to run `draw` on flowcharts that the author of the prototype did not
write.
"""

import itertools
import pathlib
import re
import sys

# The tokenizer of SPIKE-17 reads the Mermaid syntax. It lives beside the corpus.
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / "corpus"))
import measure  # noqa: E402

# The spike parser splits a name at these strings. Each one becomes a string of the same
# length, so the box keeps the width that the author's label gives it.
RESERVED = [("->", "=>"), (",", ";"), (":", ";"), ("{", "("), ("}", ")")]


class Flow(measure.Chart):
    """A chart that also keeps its links and the subgraph of each node."""

    def __init__(self, text):
        super().__init__(text)
        self.edges = []
        self.subgraphs = []  # (id, title, parent index)
        self.open = []
        self.mentioned = []  # node ids mentioned directly inside each subgraph
        self.member = {}  # node id -> subgraph index
        self.last = []

    def node(self):
        nid = super().node()
        if nid is not None:
            self.last.append(nid)
            if self.open:
                self.mentioned[self.open[-1]].append(nid)
        return nid

    def nodes(self):
        self.last = []
        return super().nodes()

    def statement(self):
        word = re.match(r"[A-Za-z_]\w*", self.text[self.pos :])
        word = word[0] if word else ""
        if word == "subgraph":
            self.pos += len(word)
            line = self.rest_of_line().strip()
            m = re.match(r"([^\s\[]+)\s*\[(.*)\]\s*$", line)
            sid, title = (m[1], m[2]) if m else (line, line)
            self.subgraphs.append((sid, title, self.open[-1] if self.open else None))
            self.mentioned.append([])
            self.open.append(len(self.subgraphs) - 1)
            return
        keyword = not re.match(r"\w*[\[\(\{>-]", self.text[self.pos + len(word) :])
        if word == "end" and keyword:
            self.rest_of_line()
            if self.open:
                # Mermaid keeps a node in the first subgraph that closes around it, so an
                # inner subgraph wins over the one around it.
                closed = self.open.pop()
                for nid in self.mentioned[closed]:
                    self.member.setdefault(nid, closed)
            return
        if (word in measure.SKIP_WORDS and keyword) or (
            word == "accDescr" and re.match(r"accDescr\s*\{", self.text[self.pos :])
        ):
            # A style line or a title: the parent skips the line.
            super().statement()
            return
        start = self.pos
        if self.nodes():
            left = self.last
            while True:
                before = self.pos
                self.skip_space(newlines=True)
                if not self.link():
                    self.pos = before
                    break
                self.skip_space(newlines=True)
                if not self.nodes():
                    break
                self.edges.extend(itertools.product(left, self.last))
                left = self.last
        self.skip_space()
        if self.pos < len(self.text) and self.text[self.pos] not in "\n;":
            self.unparsed += 1
            self.rest_of_line()
        elif self.pos == start:
            self.pos += 1


def name(raw, fallback):
    text = measure.plain(raw, True) or fallback
    for old, new in RESERVED:
        text = text.replace(old, new)
    return text


def convert(text):
    """Returns (horse source, counts), or (None, reason)."""
    flow = Flow(text).parse()
    if flow.unparsed:
        return None, f"{flow.unparsed} lines not understood"
    if flow.open:
        return None, "a subgraph is not closed"
    sids = {sid for sid, _, _ in flow.subgraphs}
    if any(a in sids or b in sids for a, b in flow.edges):
        return None, "a link ends at a subgraph"
    loops = sum(1 for a, b in flow.edges if a == b)
    edges = []
    for a, b in flow.edges:
        if a != b and (a, b) not in edges:
            edges.append((a, b))
    if not edges:
        return None, "no link between two nodes"

    linked = {n for edge in edges for n in edge}
    # A node with no link and no subgraph has no line in the subset.
    nodes = [n for n in flow.order if n in linked or n in flow.member]
    lone = len(flow.order) - len(nodes)
    # Two nodes with one label stay two things: the second gets a number.
    names, seen = {}, {}
    for nid in nodes:
        base = name(flow.labels[nid] or nid, nid)
        seen[base] = seen.get(base, 0) + 1
        names[nid] = base if seen[base] == 1 else f"{base} {seen[base]}"

    lines = [f"{names[a]} -> {names[b]}" for a, b in edges]

    def group(index, depth):
        sid, title, _ = flow.subgraphs[index]
        pad = "  " * depth
        out = [f"{pad}{name(title, sid)} {{"]
        members = [names[n] for n in nodes if flow.member.get(n) == index]
        if members:
            out.append(pad + "  " + ", ".join(members))
        for child, (_, _, parent) in enumerate(flow.subgraphs):
            if parent == index:
                out.extend(group(child, depth + 1))
        out.append(f"{pad}}}")
        return out

    for index, (_, _, parent) in enumerate(flow.subgraphs):
        if parent is None:
            lines.extend(group(index, 0))
    counts = {
        "things": len(nodes),
        "links": len(edges),
        "groups": len(flow.subgraphs),
        "self links dropped": loops,
        "lone nodes dropped": lone,
    }
    return "\n".join(lines) + "\n", counts


def main():
    if len(sys.argv) != 3:
        sys.exit("usage: mermaid.py <flowcharts directory> <output directory>")
    source, target = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    target.mkdir(parents=True, exist_ok=True)
    for old in target.glob("*.horse"):
        old.unlink()
    files = sorted(source.glob("*.mmd"))
    totals = {}
    failed = 0
    for path in files:
        horse, counts = convert(path.read_text(encoding="utf-8"))
        if horse is None:
            failed += 1
            print(f"not converted  {path.name}  {counts}")
            continue
        (target / f"{path.stem}.horse").write_text(horse, encoding="utf-8")
        for key, value in counts.items():
            totals[key] = totals.get(key, 0) + value
    print(f"flowcharts     {len(files)}")
    print(f"converted      {len(files) - failed}")
    print(f"not converted  {failed}")
    for key, value in totals.items():
        print(f"{key:15}{value}")


if __name__ == "__main__":
    main()
