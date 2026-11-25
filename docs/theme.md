# The plain theme

The plain theme is the default look of every picture. It sits inside any
document, in light mode and in dark mode. The SVG backend draws it with twelve
colour tokens and one font.

## How a page restyles it

Each token is the CSS variable `--hg-<name>`. A page that holds the SVG inline
sets a variable to change that colour. Where the page sets none, the picture
uses the value in the table. The picture follows the reader's colour scheme.
A page sets `--hg-scheme` to `light` or `dark` to force one.

## The tokens

| Token | Light | Dark | Paints |
| --- | --- | --- | --- |
| `--hg-ink` | `#1F2328` | `#E6E8EB` | Text and arrow strokes |
| `--hg-paper` | `#FFFFFF` | `#15171B` | The fill of a thing |
| `--hg-line` | `#D8DCE2` | `#343941` | The stroke of a group |
| `--hg-muted` | `#667080` | `#9AA4B2` | The stroke of a thing |
| `--hg-accent` | `#2F5FD0` | `#7AA2FF` | Arrow heads and the first owner stroke |
| `--hg-accent-ink` | `#FFFFFF` | `#0B1020` | Text on an accent fill |
| `--hg-c0` | `#F3F4F6` | `#23272E` | The fill of a group |
| `--hg-c1` | `#E1EAFB` | `#1D2B47` | The fill of the first owner |
| `--hg-c2` | `#E0F2E8` | `#17322A` | The fill of the second owner |
| `--hg-c3` | `#FAEFD6` | `#3A2F17` | The fill of the third owner |
| `--hg-s2` | `#1F7A55` | `#5FCB9B` | The stroke of the second owner |
| `--hg-s3` | `#A8621B` | `#E3A75C` | The stroke of the third owner |

The font is `--hg-font`. Its value is `Arial, "Liberation Sans", Helvetica,
sans-serif`, the face the core measures in. A theme can name another face.
Positions stay the same.

## The palette and the `owner` fact

The palette follows the fact that the lens binds to `colour`. The flow lens
binds `colour: owner`, so the palette follows the `owner` fact. The owners are numbered in the order
that their things appear in the picture. The first owner gets `--hg-c1` and
the accent stroke, the second gets `--hg-c2` and `--hg-s2`, and the third gets
`--hg-c3` and `--hg-s3`. A fourth owner starts again at the first. A thing
with no owner keeps `--hg-paper`. The class `slot-1`, `slot-2` or `slot-3`
on the thing names the slot.

## Contrast

The picture draws one rectangle of `--hg-paper` over its whole area, so link
strokes and labels sit on `--hg-paper` on any host page. Text in the picture
is `--hg-ink`. It sits on `--hg-paper`, on `--hg-c0` (a
group) or on one of `--hg-c1` to `--hg-c3`. In both modes the ratio is at
least 4.5 to 1. A test in `crates/horsegirl-core/tests/theme.rs` computes
the ratio for the reference flow and fails below that.
