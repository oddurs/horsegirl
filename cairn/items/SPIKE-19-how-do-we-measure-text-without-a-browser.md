---
id: 19
uid: 81738782-b85a-4ca7-9d07-69dcd747f0dc
title: How do we measure text without a browser?
type: spike
status: done
milestone: v0.1
assignee: Oddur Sigurdsson
created: 2026-11-25
updated: 2026-11-25
closed_at: 2026-11-25
priority: p0
risk: medium
surface: svg
effort: m
area: render
track:
- pictures
---

## Question

How does the core measure text with no browser, so that the same input gives the same SVG on every machine and in WebAssembly?

## Why it matters

Node sizes come from label widths. A wrong width makes every label overflow or float.

## Time box

Two days.

## Answer

### Conclusion

The core measures text with a bundled metrics table. The table holds the advance of each character and a pair adjustment for each pair of kerning classes, in integer font units. A label's width is the sum of its advances and its pair adjustments, converted to pixels in one step. The same integers give the same width on every machine and in WebAssembly. The core does not carry a shaping crate or a font file.

A generator program makes the table from a pinned font file and writes it as checked-in Rust source. The generator gets each advance and each pair adjustment from a shaping crate, so the table repeats what the shaper does for one and two characters. The core and its build never read the font.

The measuring face matters more than the measuring method. In the browser, a fallback face is up to 35.7 percent away from the face the table was made for. A standalone SVG cannot load a web font, because it has no external resource (0034). So:

- The core measures in Arial metrics, with Liberation Sans 2.1.5 (SIL Open Font Licence) as the source of the table. The SVG font stack is `Arial, "Liberation Sans", Helvetica, sans-serif`. Windows, macOS and iOS ship Arial, and most Linux desktops ship Liberation Sans, so most viewers draw the measured face. Helvetica has the same advances as Arial by design. These two facts are published font data; this spike did not test them. Instrument Sans, the face of the concept page, is installed almost nowhere. It would be exact only if each SVG embedded it.
- Each SVG label carries `textLength` with its measured width and `lengthAdjust="spacingAndGlyphs"` (0118). A viewer in any other face, and a theme that changes the font (0066), then keeps every label inside its box.
- A character that the table does not hold measures as one em. A label then grows rather than overflows.
- Pairs are generated for Basic Latin, Latin-1, Latin Extended-A, Greek, Cyrillic, and the dashes and quotes of General Punctuation. A pair with any other character has no adjustment. Class 0 of each matrix is reserved for "no adjustment", because every character outside those blocks takes class 0.

### Evidence

The files are in `spikes/0019-text-metrics`. `labels.txt` holds 108 labels. The first 100 are Latin: the names and link labels of `examples/checkout.horse`, typical service names, accented and Nordic letters, typographic quotes and dashes, digits, punctuation, kerning pairs such as `AVATAR` and `WAVE Yard`, and long sentences. The last 8 hold arrows, Greek and Cyrillic, such as `A→A`, `ΑΥΤΟ`, `ГАТУ` and `Тест → ΔΓ`. `measure.sh` draws each label as SVG `text` at 12 px in headless Chromium 156 and records `getComputedTextLength()` in `browser.tsv`, for Instrument Sans (pinned by `fetch-font.sh`), Liberation Sans 2.1.5 and DejaVu Sans. Two runs gave identical files. `compare` measures the same labels with each method and prints the relative error against the browser. `sizes.sh` builds a release binary for each method, native and `wasm32-unknown-unknown`, and prints its size.

Error against the browser, 12 px, regular weight. Instrument Sans has no Greek or Cyrillic glyphs, so its rows cover the 100 Latin labels. The other rows cover all 108 labels.

| Face the table is made from | Method | Mean error | Largest error | Over 1 % | Over 3 % |
| --- | --- | --- | --- | --- | --- |
| Instrument Sans | table, advances only | 1.184 % | 6.692 % | 42 | 10 |
| Instrument Sans | table, advances and pairs | 0.022 % | 0.728 % | 0 | 0 |
| Instrument Sans | shaping crate | 0.014 % | 0.061 % | 0 | 0 |
| Liberation Sans | table, advances only | 0.315 % | 6.264 % | 11 | 5 |
| Liberation Sans | table, advances and pairs | 0.024 % | 1.149 % | 1 | 0 |
| Liberation Sans | shaping crate | 0.024 % | 1.149 % | 1 | 0 |
| DejaVu Sans | table, advances and pairs | 0.012 % | 0.044 % | 0 | 0 |
| DejaVu Sans | shaping crate | 0.015 % | 0.317 % | 0 | 0 |

The remaining error of about 0.01 percent is the browser's rounding to 1/64 px. Two table errors are above 0.1 percent:

- `Łódź office` in Instrument Sans (0.728 %). The font adjusts a glyph by its context beyond one pair.
- `Тест → ΔΓ` in Liberation Sans (1.149 % narrower). The table and the shaping crate are both 114 units narrow. The font kerns the space before `Δ` by -113 units, so the gap is that one pair. The space ends a Cyrillic run and `Δ` starts a Greek run, and the browser does not apply a pair across the two runs.

The one shaping-crate error above 0.1 percent that the table does not have is `Αθήνα region` in DejaVu Sans (0.317 %). This spike did not find its cause.

The browser's own width in a fallback face, against the face the labels were measured in, on the 100 Latin labels:

| Measured in | Drawn in | Mean difference | Largest difference |
| --- | --- | --- | --- |
| Instrument Sans | Liberation Sans | 3.589 % | 12.478 % (86 labels narrower) |
| Instrument Sans | DejaVu Sans | 8.920 % | 35.660 % (all wider) |
| Liberation Sans | Instrument Sans | 3.742 % | 14.257 % |
| Liberation Sans | DejaVu Sans | 12.315 % | 28.637 % (all wider) |

Size that each method adds to a stripped release binary, with LTO, over a baseline that only reads and prints a label. One kB is 1000 bytes.

| Method | Native | Native, gzip | WebAssembly | WebAssembly, gzip |
| --- | --- | --- | --- | --- |
| table, Instrument Sans (343 characters, 188 by 171 classes) | 68.7 kB | 17.1 kB | 69.0 kB | 16.8 kB |
| table, Liberation Sans (2327 characters, 101 by 112 classes) | 43.0 kB | 9.1 kB | 43.2 kB | 9.2 kB |
| shaping crate `harfrust` 0.15.0 and the Instrument Sans file | 1216.7 kB | 593.7 kB | 1194.3 kB | 459.2 kB |
| shaping crate `harfrust` 0.15.0 and the Liberation Sans file | 1433.2 kB | 689.4 kB | 1410.8 kB | 555.8 kB |

The table is within the 3 percent budget of 0030 on every label that the table's face covers: the 108 labels in Liberation Sans and DejaVu Sans, and the 100 Latin labels in Instrument Sans. The table is 3 to 6 percent of the size of the shaping crate. The shaping crate is more accurate only where a font adjusts a glyph by more than one neighbour, which happened once, in Instrument Sans. In Liberation Sans and DejaVu Sans the shaping crate is never more accurate than the table on these labels. A list of every non-zero pair instead of class matrices holds 40438 pairs for Instrument Sans. At 12 bytes a pair, that list is about 485 kB, more than twice the 194 kB font file. Classes are necessary.

`textLength` was checked in Chromium on four labels: with `lengthAdjust="spacingAndGlyphs"`, the bounding box of a label in DejaVu Sans or Liberation Sans was within 1 px of the measured Instrument Sans width. Firefox and WebKit were not available. 0118 checks them.

To repeat the measurement:

```sh
cd spikes/0019-text-metrics
./fetch-font.sh && ./measure.sh
cargo run --release --bin compare -- browser.tsv instrument
cargo test --release --lib
mkdir -p font/system && cp /usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf font/system/
FONT=font/system/LiberationSans-Regular.ttf cargo run --release --target-dir target/liberation --bin compare -- browser.tsv arial-metrics
FONT=font/system/LiberationSans-Regular.ttf cargo test --release --target-dir target/liberation --lib
./sizes.sh
FONT=font/system/LiberationSans-Regular.ttf ./sizes.sh
```

### What this does not settle

- Arial itself was not measured, because the machine has no Arial. Liberation Sans has the advances of Arial by design. Its kerning can differ from Arial's. The advances-only error of Liberation Sans (largest 4.2 percent) bounds what kerning alone can change.
- Only the regular weight was measured. A bold label needs its own table, generated the same way.
- Height is one line: the ascender minus the descender of the face, read by the generator.
- The evidence covers Latin, Greek and Cyrillic, left to right. Liberation Sans also has Hebrew, which is right to left. A Hebrew character is outside the kerned blocks, so it gets its advance and no pairs. No Hebrew label was measured. Scripts that need shaping beyond pairs, such as Arabic and the scripts of India, are not in Liberation Sans and measure as one em for each character.
- Only 8 labels hold Greek, Cyrillic or arrows. Liberation Sans kerns Greek and Cyrillic, so these labels need the pairs: without them, `ГАТУ` is 6.3 percent wide.
- The browser does not apply a pair across a change of script, and the table does. On `Тест → ΔΓ` that makes the table 1.149 percent narrow. A label that overflows is worse than one that is wide, so 0030 can drop a pair whose two characters are in different scripts. This spike did not test that change.

### Follow-up items

- 0030 implements the table, the generator and the one-em fallback, and holds widths within 3 percent of `browser.tsv`.
- 0118 puts `textLength` on every SVG label and checks it in Chromium, Firefox and WebKit.
- 0034 and 0035 name the font stack above.

## Acceptance criteria

- [x] The Answer section states a conclusion and the evidence for it.
- [x] Follow-up items exist for the work the answer implies.
- [x] The answer compares bundled metrics tables with a shaping crate, by binary size and by error against a browser's measurement on 100 labels.

## 2026-11-25

Ran spikes/0019-text-metrics/measure.sh twice in headless Chromium 156: browser.tsv identical, 100 labels in three faces. compare browser.tsv instrument printed: table advances only mean 1.184% max 6.692%; table advances and pairs mean 0.022% max 0.728%, 0 labels over 1%; shaping crate mean 0.014% max 0.061%. With FONT=LiberationSans-Regular.ttf against the arial-metrics column: table with pairs max 0.051%, shaping max 0.051%. Fallback faces in the browser differ from the measured face by up to 35.660% (DejaVu Sans against Instrument Sans).

## 2026-11-25

Ran sizes.sh: over the baseline, the Instrument Sans table adds 68720 bytes native and 68968 bytes wasm32 (16848 gzip); the Liberation Sans table adds 21591 bytes wasm32 (7855 gzip); harfrust 0.15.0 with the Instrument Sans file adds 1216712 bytes native and 1194294 bytes wasm32 (459248 gzip).

## 2026-11-25

Checked textLength with lengthAdjust=spacingAndGlyphs in Chromium on four labels drawn in DejaVu Sans and Liberation Sans: getBBox width within 1 px of the measured Instrument Sans width. Firefox and WebKit are not installed on this machine, so 0118 carries that check.

## 2026-11-25

Follow-ups: 0030 (exists) implements the table; filed 0118 for textLength on each SVG label; notes on 0030, 0034 and 0035 record the font stack and the data path. Searched cairn for textLength, label, overflow and font first: no duplicate.

## 2026-11-25

Review round 1. Added a test that a character outside the kerned blocks has no pair: with FONT=LiberationSans-Regular.ttf, cargo test --release --lib failed before the fix ('pair → A' left -113 right 0) and passes after; it passes for Instrument Sans and DejaVu Sans. build.rs reserves class 0 for no adjustment and kerns Greek and Cyrillic. Added 8 labels with arrows, Greek and Cyrillic; measure.sh twice in Chromium 156: identical, first 100 rows unchanged. compare arial-metrics on 108 labels: table with pairs mean 0.024% max 1.149% (Тест → ΔΓ), shaping crate the same; advances only max 6.264% (ГАТУ). compare dejavu: table max 0.044%, shaping max 0.317%. sizes.sh with Liberation Sans: table adds 42976 bytes native, 43246 bytes wasm32 (9244 gzip); Instrument Sans table and both shaping builds unchanged.
