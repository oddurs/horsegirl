# Quickstart

This page takes you from nothing to one picture. You need a Rust toolchain
and about five minutes. Each command below was run, and the output is what it
printed.

## Install

There is no release yet. Build from a checkout:

```sh
git clone https://github.com/oddurs/horsegirl
cd horsegirl
cargo install --path crates/horsegirl --locked
```

Check that the binary runs:

```text
$ horsegirl --version
horsegirl 0.1.0
```

## Read the card

`horsegirl spec` prints the whole language on one page. An agent reads it
before it writes a file. The [reference](reference.md) states each rule in
full.

```text
$ horsegirl spec
horsegirl in one page

1 A name is a thing. Spaces are fine.      Web app
2 An arrow links things. Commas fan out.   A, B -> C: label
3 Braces hold parts and facts.             G { A, B }   A { owner: X }
4 A story is links in order.               story S { ... }
5 show picks a lens.                       show flow | sequence of S | lanes of S
                                           matrix | rings from T | seams by owner

-x forbids a link.  A final ? marks a guess.
Quote a name or a value that holds  , : { } @ ->  as in  "Sensors: wind" -> Hub
One fact per line. Order is free outside a story.
A line that starts with # is a comment.
End a line with  @ path:line  to cite evidence.
Run check after every edit. Apply the fix it returns.
```

This version draws only the `flow` lens. The card also names five lenses that
come later.

## Write a file

Save this as `shop.horse`:

```horsegirl
Web app, Mobile app -> API gateway
API gateway -> Orders, Payments
Orders -> Postgres
show flow
```

Each line states facts. Two things on the left of an arrow both link to the
thing on the right. The last line picks the lens.

## Check it

```text
$ horsegirl check shop.horse
shop.horse: 0 errors, 0 warnings, 0 notes
$ echo $?
0
```

## Draw it

```text
$ horsegirl draw shop.horse
horsegirl draw: wrote shop.svg
```

`shop.svg` now sits beside `shop.horse`. Its description reads `6 things, 0
groups and 5 links.` Open the file in a browser to see the picture. Add `-o -`
to print the SVG to standard output instead.

## Fix a mistake

A thing exists from its first mention, so a typo makes a new thing. Save this
as `typo.horse`:

```horsegirl
Web app, Mobile app -> API gateway
API gateway -> Orders, Payments
Orders -> Postgres
API gatway -> Postgres
show flow
```

`check` names the line, the cause and the fix, and exits 1:

```text
$ horsegirl check typo.horse
error[E0412]: `API gatway` is written once and is one edit from `API gateway`. A thing exists from its first mention, so this makes a new thing. Write `API gateway` if you mean that thing; if not, choose a name that differs by more than one character.
 --> typo.horse:4:1
  |
4 | API gatway -> Postgres
  | ^^^^^^^^^^
  = safe fix: Replace `API gatway` with `API gateway`.

typo.horse: 1 error, 0 warnings, 0 notes
$ echo $?
1
```

An agent adds `--json`. It gets one JSON object on one line for each
diagnostic, and nothing else. [Diagnostics](diagnostics.md) states the shape.

`fix` applies every safe fix and changes no other byte:

```text
$ horsegirl fix typo.horse
typo.horse: 1 safe fix applied; left: 0 errors, 0 warnings, 0 notes
$ cat typo.horse
Web app, Mobile app -> API gateway
API gateway -> Orders, Payments
Orders -> Postgres
API gateway -> Postgres
show flow
```

Run `check` once more. It is clean, so `draw` gives a picture.

## Next

- [Reference](reference.md): the five rules, each command and each code.
- [Lenses](lenses.md): what a lens binds, and what the flow lens draws.
- [Diagnostics](diagnostics.md): the JSON form and the full catalogue.
