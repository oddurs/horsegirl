# The benchmark

The benchmark measures one claim: a model writes horsegirl correctly from the
language card alone.

Each task describes a system in prose. The runner gives a model the card and
the description, and takes the file that the model writes. It scores three
things for each task:

1. The file is valid on the first try: `check` gives no error.
2. The file is valid after one `fix`: the safe fixes are applied, as
   `horsegirl fix` applies them, and `check` gives no error.
3. The fixed file states every fact that the checker of the task expects.

A valid file also states at least one link or `-x` rule. An empty reply and a
refusal in prose give no error, but they describe no system, so they are not
valid. A task with no reply scores as a failure.

## Tasks

`bench/tasks` holds 100 tasks. A task is one file, `NNN-slug.task`:

```
A web shop's checkout

A `Web app` sends each order to the `API gateway`. ...
---
Web app -> API gateway
API gateway -> Orders
```

The first line is the title. The description follows, up to a line that is
`---` alone. The description writes each name, label, fact key, fact value
and reason in backticks, exactly as the checker writes it. A fact may give its
key and its value in one pair of backticks, as `` `critical: yes` ``.
The checker follows the `---` line. It is a horsegirl file. Each thing, part,
fact, link, `-x` rule and `show` line in it is one expected fact. A link must
match its label and its guess mark, and a rule its reason, where the checker
states them. An answer may state more than the checker. Names, labels and
values match without regard to case.

The checker does not see the order of a story or a citation, because the model
does not hold them yet. A story task is checked by its links.

The tests hold every task to three rules: the checker is a valid file, it has
no lone thing, and the description gives each of its names, labels, facts and
reasons in backticks.

## Running a model

The runner knows a model only through an adapter. `horsegirl-bench run` runs a
command once for each task, through `sh -c`. The command reads the prompt on
standard input and writes the reply on standard output. `HORSEGIRL_TASK` holds
the id of the task. Each reply is recorded as `<id>.txt`:

```sh
cargo run -p horsegirl-bench -- run --command 'my-model-script' bench/runs/my-model
```

`horsegirl-bench prompt 001` prints the prompt of one task.

## Scoring a recorded run

`horsegirl-bench score` reads the recorded replies and needs no network. It
refuses a path that is not a directory:

```sh
cargo run -p horsegirl-bench -- score bench/runs/hand
```

The report lists the three totals, the diagnostics that the replies hit most
often, and the score of each task.

`bench/runs/hand` holds replies written by hand, not by a model. Some are
valid and some are not. They prove the scorer, and its report is a golden
file that the tests compare. No run of a real model is in the repository yet.
