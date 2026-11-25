use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use horsegirl_core::check::check;
use horsegirl_core::diagnostic::{Diagnostic, LineIndex, Severity, apply_safe};
use horsegirl_core::model::lower;
use horsegirl_core::syntax::parse;
use horsegirl_core::text::Style;
use horsegirl_core::{format, lens, svg};

// The site lists the commands from the Commands block, read to the end of
// this text, so the block stays last.
const USAGE: &str = "\
horsegirl: an agent-first diagram language

Usage: horsegirl <command> [options] [FILE]

FILE is `-` or absent for standard input. Exit codes: 0 when the file has no
error, 1 when it has one, 2 for a mistake in the command line.
Run `horsegirl <command> --help` for the options of one command.

Commands:
  spec       Print the one-page language card
  check      Print every diagnostic of a file, each with its fix
  fix        Apply every safe fix to a file
  fmt        Print a file in its one canonical form
  draw       Draw a file as SVG
  --version  Print the version
  --help     Print this help";

const SPEC_USAGE: &str = "\
Usage: horsegirl spec

Prints the language card: the whole language on one page. Read it before you
write a file.";

const CHECK_USAGE: &str = "\
Usage: horsegirl check [--json] [FILE]

Prints every diagnostic of FILE: its code, its place, what to do next and,
where one exists, the exact fix. Exits 0 when no diagnostic is an error, and
1 when one is. Warnings and notes do not change the exit code.

  --json  Print one JSON object on one line for each diagnostic, and nothing
          else. docs/diagnostics.md states the shape.";

const FIX_USAGE: &str = "\
Usage: horsegirl fix [--json] [FILE]

Applies every safe fix that check gives for FILE, and changes no other byte.
Writes FILE in place, or prints the result when the input is standard input.
Then prints to standard error each diagnostic that is left. A suggested fix
is left for you to decide. Exits 1 when an error is left.

  --json  Print each diagnostic that is left as one JSON object on one line.";

const FMT_USAGE: &str = "\
Usage: horsegirl fmt [--check] [FILE]

Prints FILE in its canonical form. With --check, prints nothing and exits 1
when the file would change. A file with an error has no canonical form: fmt
names each error and exits 1. A bare name that needs quotes and holds a `\"`
has none either: fmt names it and exits 1.";

const DRAW_USAGE: &str = "\
Usage: horsegirl draw [--to svg] [-o OUT] [--json] [FILE]

Draws FILE through the lens of its first `show` line, or through flow when
it has none. Writes the picture beside FILE with the extension .svg, or to
OUT. `-o -`, or standard input with no -o, prints the picture.
A file with an error gives no picture: draw names each error and exits 1.

  --to svg  The format of the picture. svg is the only one.
  -o OUT    The path to write the picture to.
  --json    Print each diagnostic as one JSON object on one line.";

/// The size of the text of a picture, in pixels.
const STYLE: Style = Style { size: 14.0 };

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next();
    let rest: Vec<String> = args.collect();
    match command.as_deref() {
        Some("spec") => spec(&rest),
        Some("check") => run("check", CHECK_USAGE, &[], rest, |options| {
            check_command(&options)
        }),
        Some("fix") => run("fix", FIX_USAGE, &[], rest, |options| fix(&options)),
        Some("fmt") => run("fmt", FMT_USAGE, &["--check"], rest, |options| {
            fmt(&options)
        }),
        Some("draw") => run("draw", DRAW_USAGE, &["--to", "-o"], rest, |options| {
            draw(&options)
        }),
        Some("--version" | "-V") => {
            println!("horsegirl {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("horsegirl: unknown command `{other}`\n\n{USAGE}");
            ExitCode::from(2)
        }
        None => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn spec(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => {
            print!("{}", horsegirl_core::SPEC);
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") => {
            println!("{SPEC_USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("horsegirl spec: unexpected `{other}`\n\n{SPEC_USAGE}");
            ExitCode::from(2)
        }
    }
}

/// The command line of one command, read.
struct Options {
    json: bool,
    /// `--check`, the one switch of `fmt`.
    check: bool,
    /// The value of each option that takes one, such as `-o`.
    values: Vec<(&'static str, String)>,
    input: Input,
}

impl Options {
    fn value(&self, name: &str) -> Option<&str> {
        let found = self.values.iter().find(|(key, _)| *key == name);
        found.map(|(_, value)| value.as_str())
    }
}

struct Input {
    /// The file, or `None` for standard input.
    path: Option<PathBuf>,
    /// What a diagnostic calls the file.
    label: String,
    source: String,
}

/// Reads the command line of `name` and its input, then runs `command`.
///
/// Every command but `fmt` takes `--json`. `extra` names the other options of
/// the command: `--check` is a switch, and every other one takes a value.
fn run(
    name: &str,
    usage: &str,
    extra: &[&'static str],
    args: Vec<String>,
    command: impl FnOnce(Options) -> ExitCode,
) -> ExitCode {
    let usage_error = |message: String| {
        eprintln!("horsegirl {name}: {message}\n\n{usage}");
        ExitCode::from(2)
    };
    let mut json = false;
    let mut check = false;
    let mut values = Vec::new();
    let mut file: Option<String> = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("{usage}");
                return ExitCode::SUCCESS;
            }
            "--json" if name != "fmt" => json = true,
            "--check" if extra.contains(&"--check") => check = true,
            option if extra.contains(&option) => {
                let key = extra.iter().find(|known| **known == option).copied();
                let (Some(key), Some(value)) = (key, args.next()) else {
                    return usage_error(format!("`{option}` needs a value"));
                };
                if values.iter().any(|(seen, _)| *seen == key) {
                    return usage_error(format!("`{option}` is given twice"));
                }
                // Checked before the input is read, so a mistake in the
                // command line is named first.
                if key == "--to" && value != "svg" {
                    return usage_error(format!(
                        "`--to {value}` is not a format; svg is the only one"
                    ));
                }
                values.push((key, value));
            }
            option if option.starts_with('-') && option != "-" => {
                return usage_error(format!("unknown option `{option}`"));
            }
            _ if file.is_some() => return usage_error("one file at a time".to_owned()),
            _ => file = Some(arg),
        }
    }
    let path = file.filter(|file| file != "-").map(PathBuf::from);
    let label = path
        .as_ref()
        .map_or_else(|| "<stdin>".to_owned(), |path| path.display().to_string());
    let read = if let Some(path) = &path {
        std::fs::read_to_string(path)
    } else {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text).map(|_| text)
    };
    let source = match read {
        Ok(source) => source,
        Err(error) => {
            eprintln!("horsegirl {name}: cannot read {label}: {error}");
            return ExitCode::from(2);
        }
    };
    command(Options {
        json,
        check,
        values,
        input: Input {
            path,
            label,
            source,
        },
    })
}

/// The diagnostics of `input`: one JSON object on each line, or the text form
/// with a blank line between two diagnostics.
fn report(input: &Input, diagnostics: &[Diagnostic], json: bool) -> String {
    let lines = LineIndex::new(&input.source);
    if json {
        let objects = diagnostics.iter().map(|d| d.to_json(&lines) + "\n");
        objects.collect()
    } else {
        let blocks = diagnostics.iter().map(|d| d.render(&lines, &input.label));
        blocks.collect::<Vec<_>>().join("\n")
    }
}

fn errors(diagnostics: &[Diagnostic]) -> usize {
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity() == Severity::Error);
    errors.count()
}

/// The count of each severity, as a person reads it.
fn summary(diagnostics: &[Diagnostic]) -> String {
    let count = |severity: Severity| {
        let found = diagnostics.iter().filter(|d| d.severity() == severity);
        let n = found.count();
        let word = severity.word();
        if n == 1 {
            format!("1 {word}")
        } else {
            format!("{n} {word}s")
        }
    };
    format!(
        "{}, {}, {}",
        count(Severity::Error),
        count(Severity::Warning),
        count(Severity::Note)
    )
}

fn exit(diagnostics: &[Diagnostic]) -> ExitCode {
    if errors(diagnostics) > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn check_command(options: &Options) -> ExitCode {
    let input = &options.input;
    let diagnostics = check(&input.source);
    print!("{}", report(input, &diagnostics, options.json));
    if !options.json {
        let separator = if diagnostics.is_empty() { "" } else { "\n" };
        println!("{separator}{}: {}", input.label, summary(&diagnostics));
    }
    exit(&diagnostics)
}

fn fix(options: &Options) -> ExitCode {
    let input = &options.input;
    let fixed = apply_safe(&input.source, &check(&input.source));
    if let Some(path) = &input.path {
        if fixed.text != input.source
            && let Err(error) = std::fs::write(path, &fixed.text)
        {
            eprintln!("horsegirl fix: cannot write {}: {error}", input.label);
            return ExitCode::from(2);
        }
    } else {
        print!("{}", fixed.text);
    }
    let left = Input {
        path: None,
        label: input.label.clone(),
        source: fixed.text,
    };
    let diagnostics = check(&left.source);
    eprint!("{}", report(&left, &diagnostics, options.json));
    if !options.json {
        let noun = if fixed.applied == 1 { "fix" } else { "fixes" };
        let separator = if diagnostics.is_empty() { "" } else { "\n" };
        eprintln!(
            "{separator}{}: {} safe {noun} applied; left: {}",
            left.label,
            fixed.applied,
            summary(&diagnostics)
        );
    }
    exit(&diagnostics)
}

fn fmt(options: &Options) -> ExitCode {
    let input = &options.input;
    let label = &input.label;
    let text = match format::format(&input.source) {
        Ok(text) => text,
        Err(errors) => {
            eprint!("{}", report(input, &errors, false));
            return ExitCode::FAILURE;
        }
    };
    // The formatter returns such a file as it is. Without this report, --check
    // would pass a file that is in no canonical form.
    if let Some(name) = format::unquotable_name(&input.source) {
        eprintln!(
            "horsegirl fmt: {label} has no canonical form: the name `{name}` needs quotes, and quotes cannot hold a `\"`. Rename it without the `\"`."
        );
        return ExitCode::FAILURE;
    }
    if !options.check {
        print!("{text}");
    } else if text != input.source {
        eprintln!("horsegirl fmt: {label} would change");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn draw(options: &Options) -> ExitCode {
    let input = &options.input;
    let diagnostics = check(&input.source);
    eprint!("{}", report(input, &diagnostics, options.json));
    let count = errors(&diagnostics);
    if count > 0 {
        let s = if count == 1 { "" } else { "s" };
        eprintln!(
            "\nhorsegirl draw: {} has {count} error{s}, so no picture was drawn. Fix each error above and draw again.",
            input.label
        );
        return ExitCode::FAILURE;
    }
    let model = lower(&parse(&input.source).tree);
    let name = model
        .shows
        .first()
        .map_or("flow", |show| show.lens.as_str());
    // The checks accept every lens of the language card; this version draws
    // fewer, so a file can be clean and still name a lens with no solver.
    let Some(lens) = lens::find(name) else {
        let drawn: Vec<&str> = lens::DRAWN.iter().map(|lens| lens.name).collect();
        eprintln!(
            "horsegirl draw: `show {name}` names a lens that this version does not draw. It draws: {}. Change the first `show` line to one of those.",
            drawn.join(", ")
        );
        return ExitCode::FAILURE;
    };
    let title = input.path.as_deref().and_then(Path::file_stem).map_or_else(
        || "diagram".to_owned(),
        |stem| stem.to_string_lossy().into_owned(),
    );
    let picture = svg::draw(&lens::draw(&model, lens, STYLE), &title);
    let out = match (options.value("-o"), &input.path) {
        (Some("-"), _) | (None, None) => None,
        (Some(out), _) => Some(PathBuf::from(out)),
        (None, Some(path)) => Some(path.with_extension("svg")),
    };
    let Some(out) = out else {
        print!("{picture}");
        return ExitCode::SUCCESS;
    };
    if input.path.as_ref() == Some(&out) {
        eprintln!(
            "horsegirl draw: the picture would overwrite {}; name another path with -o",
            input.label
        );
        return ExitCode::from(2);
    }
    if let Err(error) = std::fs::write(&out, picture) {
        eprintln!("horsegirl draw: cannot write {}: {error}", out.display());
        return ExitCode::from(2);
    }
    if !options.json {
        eprintln!("horsegirl draw: wrote {}", out.display());
    }
    ExitCode::SUCCESS
}
