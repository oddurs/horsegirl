#!/usr/bin/env python3
"""Convert the Mermaid corpus to horsegirl, so that CI can draw it.

Run from anywhere: python3 -I -B corpus/convert.py
It reads corpus/flowcharts/*.mmd and writes corpus/horse/NNNN.horse.
With --check it writes nothing, and fails when a file in corpus/horse is not
what the script writes today.

This is a stand-in for `horsegirl convert` (item 0064). It keeps the
things, the links, their labels and the subgraphs, and drops styles, shapes
and arrow kinds. It reads Mermaid with the parser of measure.py, so that the
two scripts agree on what the corpus holds.
"""

import importlib.util
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE / "horse"

# python3 -I leaves this directory off the path, so load measure.py by its path.
_spec = importlib.util.spec_from_file_location("measure", HERE / "measure.py")
measure = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(measure)


class Chart(measure.Chart):
    """The parser of measure.py, which also records links and subgraphs."""

    def __init__(self, text):
        super().__init__(text)
        self.edges = []  # (from id, to id, raw label or None)
        self.parent = {}  # node id or subgraph id -> subgraph id
        self.subgraphs = {}  # subgraph id -> raw title, in order of opening
        self.stack = []
        self.previous = []
        self.linked = False
        self.label = None
        self.found = []

    def node(self):
        nid = super().node()
        if nid is not None:
            self.found.append(nid)
            if self.stack and nid not in self.parent and nid != self.stack[-1]:
                self.parent[nid] = self.stack[-1]
        return nid

    def nodes(self):
        self.found = []
        if not super().nodes():
            return False
        if self.linked:
            for a in self.previous:
                for b in self.found:
                    self.edges.append((a, b, self.label))
        self.previous = self.found
        self.linked = False
        return True

    def link(self):
        before = len(self.edge_labels)
        if not super().link():
            return False
        self.linked = True
        self.label = self.edge_labels[-1] if len(self.edge_labels) > before else None
        return True

    def statement(self):
        self.previous = []
        self.linked = False
        rest = self.text[self.pos :]
        word = re.match(r"[A-Za-z_]\w*", rest)
        word = word[0] if word else ""
        if word == "subgraph":
            self.pos += len(word)
            line = self.rest_of_line().strip()
            m = re.match(r"([^\s\[]+)\s*\[(.*)\]\s*$", line)
            sid, title = (m[1], m[2]) if m else (measure.unquote(line), line)
            if self.stack and sid not in self.parent:
                self.parent[sid] = self.stack[-1]
            self.subgraphs.setdefault(sid, title)
            self.stack.append(sid)
            return
        if word == "end" and not re.match(r"\w*[\[\(\{>-]", rest[len(word) :]):
            if self.stack:
                self.stack.pop()
        super().statement()


def quoted(text, label=False):
    """A name or a label, in quotes when a bare one cannot hold it (ADR-119).

    A bare name that ends in `?` at the end of a link gives W0111, whose fix
    makes the `?` a guess mark. A decision node is no guess, so the quotes
    keep the `?` in the name."""
    text = text.replace('"', "'")
    needs = measure.causes(text) or (not label and text.endswith("?"))
    return f'"{text}"' if needs else text


def convert(source):
    chart = Chart(source).parse()
    names = {}
    for sid, raw in chart.subgraphs.items():
        names[sid] = measure.plain(raw, True) or sid
    for nid in chart.order:
        if nid not in names:
            raw = chart.labels[nid]
            names[nid] = (measure.plain(raw, True) if raw is not None else "") or nid
    lines = []
    linked = set()
    for a, b, raw in chart.edges:
        label = measure.plain(raw) if raw is not None else ""
        line = f"{quoted(names[a])} -> {quoted(names[b])}"
        lines.append(f"{line}: {quoted(label, label=True)}" if label else line)
        linked.update((a, b))
    held = set(chart.parent)
    for nid in chart.order:
        if nid not in linked and nid not in held and nid not in chart.subgraphs:
            lines.append(quoted(names[nid]))
    parts = {sid: [] for sid in chart.subgraphs}
    for member in [*chart.subgraphs, *chart.order]:
        sid = chart.parent.get(member)
        if sid is not None and member not in parts[sid]:
            parts[sid].append(member)
    if parts:
        lines.append("")
    for sid, members in parts.items():
        inner = ", ".join(quoted(names[m]) for m in members)
        lines.append(f"{quoted(names[sid])} {{ {inner} }}" if inner else quoted(names[sid]))
    return "\n".join(lines) + "\n"


def main():
    check = sys.argv[1:] == ["--check"]
    if sys.argv[1:] and not check:
        sys.exit("usage: python3 corpus/convert.py [--check]")
    files = sorted((HERE / "flowcharts").glob("*.mmd"))
    if not files:
        sys.exit(f"convert: no flowcharts in {HERE / 'flowcharts'}")
    wanted = {}
    for path in files:
        header = f"# Converted from corpus/flowcharts/{path.name} by corpus/convert.py.\n"
        wanted[path.stem + ".horse"] = header + convert(path.read_text(encoding="utf-8"))
    if check:
        present = {path.name for path in OUT.glob("*.horse")}
        stale = sorted(
            name for name in present | set(wanted)
            if name not in wanted or name not in present
            or (OUT / name).read_text(encoding="utf-8") != wanted[name]
        )
        if stale:
            sys.exit(
                f"convert: {len(stale)} files in corpus/horse differ from the script, "
                f"first {stale[0]}. Run python3 corpus/convert.py and commit the result."
            )
        return
    OUT.mkdir(exist_ok=True)
    for old in OUT.glob("*.horse"):
        if old.name not in wanted:
            old.unlink()
    for name, text in wanted.items():
        (OUT / name).write_text(text, encoding="utf-8")
    print(f"convert: wrote {len(wanted)} files to corpus/horse")


if __name__ == "__main__":
    main()
