# Reference

This page states the language and the commands of this version. It describes
what the program does today. [Quickstart](quickstart.md) is the short way in.

Every `horsegirl` example on this page is a file in
`crates/horsegirl-core/tests/golden`, and a test fails when the page and the
file differ.

## The five rules

The language card, `horsegirl spec`, states five rules. This section states
each one in full.

A file is a list of lines. Order does not matter outside a story. A line that
starts with `#` is a comment.

### 1. A name is a thing

A name is a bare run of words. Spaces are fine: `Web app` is one name. A thing
exists from its first mention. There is no separate declaration and no id.

A name cannot hold a reserved character: `,` `:` `{` `}` `@` or `->`. Write
such a name in double quotes, as in `"Sensors: wind"`. The bare name is the
normal form, so `fmt` removes quotes that a name does not need.

```horsegirl
"Sensors: wind", "a -> b" -> "Hub, main": "GET /users/{id}"
show flow
```

A name holds `-x` only as a part of a longer word: `Dash-x` is a name. `-x` is
a rule only as a whole word.

### 2. An arrow is a link

`A -> B` links `A` to `B`. A comma fans out on either side:
`Web app, Mobile app -> API gateway` states two links. Words after a colon
label the link.

A label runs to the end of the line. It can hold a comma, a colon and an
arrow with no quotes. A `{`, a `}` or an `@` in a label needs double quotes.

```horsegirl
# A label runs to the end of the line.
Web app -> API gateway: POST /checkout, then 200 -> ok
API gateway -> Dash-x
Dash-x -> Orders: "returns {id}"
show flow
```

### 3. `-x` is a link that must not exist

`A -x B` is a rule. It is not a link and the picture does not draw it.
`check` reports `E0500` for a link that breaks a rule in the same direction.

```horsegirl
Web app -> Postgres
Web app -x Postgres
show flow
```

### 4. A final `?` marks a guess

A `?` is a guess only as the final whole word of a statement. A link that ends
in `?` in every statement that states it is a guess, and `check` gives the note
`N0200` so that you confirm it against the system.

```horsegirl
Web app -> API gateway: POST /checkout
Web app -x Postgres: never directly
API gateway -> Cache ?
show flow
```

### 5. Braces hold what is inside and what is known

A name followed by braces is a group. The braces hold the things inside it.
A group nests, up to 64 levels deep. Deeper braces give `E0007`. A thing is in
one group, and a group is never inside itself. A thing in the braces of a
second group, or of a group inside it, gives `E0420`. So does a part in the
braces after several names: those braces hold facts only.

A `key: value` pair is a fact, and it is a fact only inside braces. A fact
applies to the things that stand before the brace. Write one fact on each
line. The flow lens reads the `owner` fact and gives each owner a colour.

```horsegirl
Services { Orders, Payments { owner: Commerce } }
Notify { owner: Growth }
Web app -> Orders
Orders -> Payments
Payments -> Notify
show flow
```

A link that starts or ends at a group is not drawn yet.

### Stories

A story is links in the order they happen.

```horsegirl
Web app, Mobile app -> API gateway
API gateway -> Payments
Web app, Mobile app -x Postgres: never directly

Services {
  Orders, Payments { owner: Commerce }
  Notify { owner: Growth }
}
API gateway, Postgres, Events { owner: Platform }
Web app, Mobile app { owner: Apps }
Email provider { owner: External }

story checkout {
  Web app -> API gateway: POST /checkout
  API gateway -> Orders: create
  Orders -> Payments: charge
  Orders -> Postgres: save
  Payments -> Notify: paid
  Notify -> Events: publish
  Events -> Email provider: send
}

show flow
```

This version parses a story and does not order its links. `check` gives
`W0010` and you need not change the file.

A line that holds `,` `:` `{` or `->` and starts with `show` or `story` is not a
`show` line or a story. It is a link.

### Evidence

End a line with `@`, a path and a line number to cite evidence. A citation is
a space, `@`, a path with no space inside, a colon and a line number, as in
` @ src/repo.ts:17`.

```horsegirl
Orders -> Postgres: save  @ src/repo.ts:17
show flow
```

### The `show` line

`show` and a lens name pick the lens. This version draws `flow` only. The card
names `sequence`, `lanes`, `matrix`, `rings` and `seams` for later versions.
`draw` refuses a lens that it cannot draw with `E0302`, and names the lenses that it can.
`show flow` needs at least one link, or `check` gives `E0301`.

The first `show` line picks the lens. A file with no `show` line draws as
flow. [Lenses](lenses.md) states what the flow lens does.

## The commands

Every command reads standard input when the file is `-` or absent. Every
command exits 0 when the file has no error, 1 when it has one, and 2 for a
mistake in the command line. A warning or a note does not change the code.
`horsegirl <command> --help` names the options of one command.

### spec

`horsegirl spec` prints the language card. Read it before you write a file.

### check

`horsegirl check [--json] [FILE]` prints every diagnostic: its code, its
place, what to do next and, where one exists, the exact fix. The last line
counts them.

```text
$ horsegirl check typo.horse
error[E0412]: `API gatway` is written once and is one edit from `API gateway`. A thing exists from its first mention, so this makes a new thing. Write `API gateway` if you mean that thing; if not, choose a name that differs by more than one character.
 --> typo.horse:4:1
  |
4 | API gatway -> Postgres
  | ^^^^^^^^^^
  = safe fix: Replace `API gatway` with `API gateway`.

typo.horse: 1 error, 0 warnings, 0 notes
```

With `--json` the command prints one JSON object on one line for each
diagnostic, and nothing else. [Diagnostics](diagnostics.md) states the shape.

```text
$ horsegirl check --json typo.horse
{"code":"E0412","fix":{"confidence":"safe","description":"Replace `API gatway` with `API gateway`.","edits":[{"span":{"end":{"column":11,"line":4},"start":{"column":1,"line":4}},"text":"API gateway"}]},"message":"`API gatway` is written once and is one edit from `API gateway`. A thing exists from its first mention, so this makes a new thing. Write `API gateway` if you mean that thing; if not, choose a name that differs by more than one character.","severity":"error","span":{"end":{"column":11,"line":4},"start":{"column":1,"line":4}}}
```

### fix

`horsegirl fix [--json] [FILE]` applies every safe fix and changes no other
byte. It checks again and applies each safe fix that appears, until none is
left. It writes the file in place, or prints the result for standard input.
Then it prints each diagnostic that is left to standard error. A suggested fix
is left for you to decide. It exits 1 when an error is left.

```text
$ horsegirl fix typo.horse
typo.horse: 1 safe fix applied; left: 0 errors, 0 warnings, 0 notes
```

### fmt

`horsegirl fmt [--check] [FILE]` prints the file in its one canonical form: one
statement on each line, one space around an arrow, and quotes only on a name
that needs them. Comments stay. With `--check` it prints nothing and exits 1
when the file would change.

A file with an error has no canonical form. `fmt` names each error and exits 1.
A bare name that needs quotes and holds a `"` has none either.

```text
$ printf 'A -> -> B\n' | horsegirl fmt
error[E0006]: A name is missing before `->`. Write the name there.
 --> <stdin>:1:6
  |
1 | A -> -> B
  |      ^^

$ echo $?
1
```

### draw

`horsegirl draw [--to svg] [-o OUT] [--json] [FILE]` draws the file through the
lens of its first `show` line. It writes the picture beside the file with the
extension `.svg`, or to `OUT`. `-o -`, or standard input with no `-o`, prints
the picture. SVG is the only format. A file with an error gives no picture:
`draw` names each error and exits 1.

```text
$ horsegirl draw shop.horse
horsegirl draw: wrote shop.svg
```

`--json` prints each diagnostic as one JSON object on one line.

## The diagnostics

Every diagnostic has a stable code. `E` is an error, `W` is a warning and `N`
is a note. Only an error changes the exit code. A diagnostic names the line and
the cause. Where a safe replacement exists, it carries the replacement as a
fix.

[Diagnostics](diagnostics.md) holds the catalogue, made from the source. The
table below says what to do for each code. A test fails when a code is missing
from it.

| Code | What to do |
| --- | --- |
| `E0001` | Remove the stray `@` or the extra word. |
| `E0002` | Close the `{` with a `}`. |
| `E0003` | Remove the `}` or open a group before it. |
| `E0004` | Write the name, the value or the label in double quotes. The fix does it. |
| `E0005` | Close the double quote on the same line. |
| `E0006` | Write the name that is missing. |
| `E0007` | Nest the braces less than 64 deep. |
| `E0008` | Write `show` and a lens, and `of`, `from` or `by` and a name when the lens needs one. |
| `E0009` | Put a fact inside braces, and keep names and facts on separate lines. |
| `E0010` | Write each link on its own line. The fix does it. |
| `E0300` | Write a lens that exists. The fix replaces a name that is one edit away. |
| `E0301` | Add a link, or remove the `show` line. |
| `E0302` | Change the first `show` line to a lens that this version draws. The message names them. |
| `E0412` | Write the name that you meant. The fix does it when only one name is near. |
| `E0420` | Keep the thing in one group, and write a part in the braces after one name. The suggested fix removes the later name; apply it only when the two mentions mean one thing. |
| `E0500` | Remove the link, or remove the rule if it no longer holds. |
| `N0101` | Write the canonical form. The fix does it. |
| `N0200` | Confirm the guess against the system, then remove the final `?`. |
| `W0010` | Nothing. This version does not order a story. |
| `W0107` | Link the thing, or remove it. |
| `W0108` | Write the whole name in double quotes, as in `"GET /users/{id}"`. |

These files each show one mistake:

```horsegirl
Web app -> API@v2
```

```horsegirl
Web app -> Postgres
show flwo
```

```horsegirl
Web app
show flow
```
