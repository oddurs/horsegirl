# The Mermaid flowchart corpus

This directory holds real Mermaid flowcharts from public GitHub repositories.
The grammar spike SPIKE-17 measured names against them. Later items use them
to test the parser, the converter and the layout.

## What is here

- `flowcharts/NNNN.mmd`: one flowchart in each file, as its author wrote it.
- `manifest.tsv`: for each file, the source repository, the path, the first
  line, the commit, the SPDX licence identifier, the path of the licence file
  in that repository, the licence text in this directory, and a permanent URL.
- `licences/<SPDX>-<hash>.txt`: the licence file of a source repository at
  that commit, stored once for all repositories that carry the same text. An
  Apache-2.0 repository with a `NOTICE` file has the notice at the end of its
  licence text.
- `measure.py`: the measurement that SPIKE-17 quotes.
- `convert.py`: writes `horse/NNNN.horse` from `flowcharts/NNNN.mmd`.
- `horse/NNNN.horse`: each flowchart in horsegirl, as `convert.py` writes it.

## How the files were chosen

1. GitHub code search found `.mmd` files and Markdown files that contain
   `flowchart` or `graph` with the directions `TD`, `LR` and `TB`.
2. The repositories were shuffled with a fixed seed.
3. A repository was used only when it is not a fork and GitHub reports one of
   these licences for it: MIT, MIT-0, Apache-2.0, BSD-2-Clause, BSD-3-Clause,
   ISC, 0BSD, Unlicense, CC0-1.0, CC-BY-4.0, Zlib, BSL-1.0.
4. A flowchart was used only when it has at least three links, is at most
   6000 bytes, and is not a copy of a flowchart already taken. At most two
   flowcharts come from one repository.

A Markdown file gives the body of its fenced `mermaid` block. Line endings are
LF, and blank lines at the start and the end are removed. Nothing else changes.

## Licences

Each file stays under the licence of its source repository, named in
`manifest.tsv`. A file in `horse/` is under the licence of the flowchart of the
same number. The horsegirl licence does not cover this directory. The
licence check uses the licence that GitHub reports for the whole repository. A
repository can put one directory under a different licence; if you find one,
remove its rows and files.

## Measure

```sh
python3 corpus/measure.py
```

The script uses only the Python standard library. It prints the same report on
every machine.

## Convert and draw

```sh
python3 corpus/convert.py
cargo run -p horsegirl --example corpus
```

`convert.py` keeps the things, the links, their labels and the subgraphs. It
drops styles, shapes and the kind of each arrow. A name or a label that holds a
reserved character is written in quotes. `scripts/task lint` fails when a file
in `horse/` differs from what the script writes, so commit the files after you
change the script.

The example draws each file with the flow lens and counts the files whose
picture has no overlap of any kind and draws every link. `scripts/task test` runs it, so CI prints
the count. The run fails when the count is below `BASELINE` in
`crates/horsegirl/examples/corpus.rs`. A file with an error in `check` gives no
picture, so it is not clean. A picture that leaves out a link of the file is
not clean either.
