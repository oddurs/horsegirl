# Diagnostics

A diagnostic is what horsegirl says about a file. Each one has a stable code, a
place, a message that says what to do next and, where one exists, the exact
replacement. horsegirl prints a diagnostic in two forms: text for a terminal,
and one JSON object on one line for an agent.

## The JSON form

This is the line for the source `Web app -> API@v2`:

```json
{"code":"E0004","fix":{"confidence":"safe","description":"Write the name in double quotes.","edits":[{"span":{"end":{"column":18,"line":1},"start":{"column":12,"line":1}},"text":"\"API@v2\""}]},"message":"`API@v2` holds a character that a bare name cannot hold. Write the name in double quotes. A citation is a space, `@`, a path and a line number at the end of the line, as in ` @ src/app.ts:17`.","severity":"error","span":{"end":{"column":18,"line":1},"start":{"column":12,"line":1}}}
```

A test compares this line with what the code prints.

The keys of every object are sorted by name. The line holds no space between
tokens and no line break, so it is the same bytes that `serde_json` prints for
the same value.

A **diagnostic** has five keys. Each key is always present.

| Key | Type | Meaning |
| --- | --- | --- |
| `code` | string | `E`, `W` or `N` and four digits. Stable across releases. |
| `fix` | fix or `null` | The replacement. `null` when there is none. |
| `message` | string | Names the cause and says what to do next. |
| `severity` | `"error"`, `"warning"` or `"note"` | Agrees with the first letter of `code`. |
| `span` | span | Where the cause is. |

A **span** is `{"end": position, "start": position}`. `end` is exclusive. A
span can be empty: then it is a place between two characters, such as the end
of a line where a name is missing.

A **position** is `{"column": n, "line": n}`. Both start at 1. A column counts
Unicode scalar values, not bytes and not UTF-16 units. Only a line feed starts
a new line.

A **fix** has three keys.

| Key | Type | Meaning |
| --- | --- | --- |
| `confidence` | `"safe"` or `"suggested"` | `safe`: a tool applies it with no question. `suggested`: an agent decides. |
| `description` | string | What the fix does, in one sentence. |
| `edits` | array of edit | One or more edits. |

An **edit** is `{"span": span, "text": string}`. The text replaces the span,
and an empty text deletes it. Every span refers to the original source. The
edits of one fix do not overlap, and they are sorted by start.

The diagnostics of one file are sorted by the start of their span and then by
code. No two fixes of one file overlap, so a tool can apply every fix of one
run. When two fixes would overlap, the one that starts first keeps its fix and
the other has `null`. The first is the outer one: in `A { B @ x } -> C`, quotes
around the whole name repair the block inside it too.

## The text form

The text form names the severity, the code and the message, then the place,
the source line and a mark under the span. A fix follows on its own line.

```text
error[E0004]: `API@v2` holds a character that a bare name cannot hold. Write the name in double quotes. A citation is a space, `@`, a path and a line number at the end of the line, as in ` @ src/app.ts:17`.
 --> example.horse:1:12
  |
1 | Web app -> API@v2
  |            ^^^^^^
  = safe fix: Write the name in double quotes.
```

The marks stop at the end of the line where the span starts. An empty span
gets one mark.

## The codes

A code keeps its meaning in every release. A code that goes out of use is not
given to a new mistake. The table below is generated from
`crates/horsegirl-core/src/codes.rs`, and a test fails when the source uses a
code that the table does not hold. To regenerate it, run
`HORSEGIRL_BLESS=1 cargo test -p horsegirl-core --test diagnostic`.

<!-- catalogue: start -->
| Code | Severity | Meaning |
| --- | --- | --- |
| `E0001` | error | A token stands where the line cannot hold it: a second arrow, a `@` that is not a citation, or a word after the end of a fact. |
| `E0002` | error | A `{` never closes. |
| `E0003` | error | A `}` closes nothing. |
| `E0004` | error | A name, a value or a label holds a character that only a text in double quotes can hold. The fix adds the quotes. |
| `E0005` | error | A double quote opens and does not close on its line. |
| `E0006` | error | A name or a value is missing. At the end of a line, the span is empty. |
| `E0007` | error | Braces nest deeper than the parser reads. |
| `E0008` | error | A `show` line is not `show` and a lens, with `of`, `from` or `by` and a name after it. |
| `E0009` | error | A fact stands outside braces, or a line mixes names and facts. |
| `E0300` | error | A `show` line names a lens that does not exist. The message lists the lenses, and a lens one edit from a known lens gets a safe fix. |
| `E0412` | error | A name is written once and is one edit from a name written more often, so it is likely a typo that makes a new thing. The fix writes the other name. |
| `E0500` | error | A link breaks a `-x` rule in the same direction. |
| `N0200` | note | A link or a rule ends in `?` in every statement, so it is a guess. |
| `W0010` | warning | A story is parsed, and this version does not order its links. |
| `W0107` | warning | A thing has no link and no rule, so it stands alone in every picture. A group needs no link of its own. |
<!-- catalogue: end -->
