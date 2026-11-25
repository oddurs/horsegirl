//! Each command run as a process: its exit code, its standard output and its
//! standard error, and the files it writes.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn horsegirl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_horsegirl"))
}

/// Runs `horsegirl` with `args` and `input` on standard input.
fn run(args: &[&str], input: &str) -> Output {
    let mut child = horsegirl()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary runs");
    let written = child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(input.as_bytes());
    // A command that stops at a usage mistake reads no input and closes the pipe.
    match written {
        Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => {}
        written => written.expect("the input is written"),
    }
    child.wait_with_output().expect("the binary ends")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// An empty directory of its own for one test.
fn scratch(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the directory is made");
    dir
}

/// Three errors, each with a safe fix, and lines that need none.
const FIXABLE: &str = "# the shop\nWeb app -> Payments\nPayments -> Bank\nOrders -> Paymnts\nC -> API@v2\nshow flwo\n";
const FIXED: &str = "# the shop\nWeb app -> Payments\nPayments -> Bank\nOrders -> Payments\nC -> \"API@v2\"\nshow flow\n";

#[test]
fn every_help_fits_on_one_screen() {
    for command in [
        &[][..],
        &["spec"],
        &["check"],
        &["fix"],
        &["fmt"],
        &["draw"],
    ] {
        let out = run(&[command, &["--help"]].concat(), "");
        assert_eq!(out.status.code(), Some(0), "{command:?}");
        let help = text(&out.stdout);
        assert!(help.starts_with("Usage: ") || command.is_empty(), "{help}");
        assert!(help.lines().count() <= 24, "{command:?}: {help}");
        let widest = help.lines().map(|line| line.chars().count()).max();
        assert!(widest <= Some(80), "{command:?}: {help}");
    }
}

#[test]
fn spec_prints_the_card() {
    let out = run(&["spec"], "");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(text(&out.stdout), horsegirl_core::SPEC);
}

#[test]
fn spec_with_an_argument_exits_2() {
    let out = run(&["spec", "flow"], "");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(text(&out.stdout), "");
    assert!(text(&out.stderr).contains("`flow`"));
}

#[test]
fn an_unknown_command_exits_2_and_names_it() {
    let out = horsegirl().arg("gallop").output().expect("the binary runs");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("`gallop`"));
}

mod fmt {
    use super::{horsegirl, text};
    use std::process::Output;

    fn run(args: &[&str], input: &str) -> Output {
        super::run(&[&["fmt"], args].concat(), input)
    }

    #[test]
    fn prints_the_canonical_form() {
        let out = run(&[], "A->B:  pay\n");
        assert!(out.status.success());
        assert_eq!(text(&out.stdout), "A -> B: pay\n");
    }

    #[test]
    fn reads_a_file() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/checkout.horse");
        let out = horsegirl()
            .args(["fmt", path])
            .output()
            .expect("the binary runs");
        assert!(out.status.success());
        assert_eq!(
            text(&out.stdout),
            include_str!("../../../examples/checkout.horse")
        );
    }

    #[test]
    fn check_exits_0_for_a_formatted_file() {
        let out = run(&["--check"], "A -> B\n");
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(text(&out.stdout), "");
    }

    #[test]
    fn check_exits_1_when_the_file_would_change() {
        let out = run(&["--check"], "A->B\n");
        assert_eq!(out.status.code(), Some(1));
        assert_eq!(text(&out.stdout), "");
        assert!(text(&out.stderr).contains("would change"));
    }

    #[test]
    fn an_error_in_the_file_exits_1_and_names_the_line() {
        let out = run(&[], "A -> B\nA -> -> B\n");
        assert_eq!(out.status.code(), Some(1));
        assert_eq!(text(&out.stdout), "");
        let stderr = text(&out.stderr);
        assert!(stderr.starts_with("error[E0006]"), "{stderr}");
        assert!(stderr.contains("--> <stdin>:2:"), "{stderr}");
    }

    #[test]
    fn a_name_that_quotes_cannot_hold_exits_1_and_is_named() {
        for input in ["A { k: x\"y ?, a: b }\nC->D\n", "A { #a\"b }\nC->D\n"] {
            for args in [&["--check"][..], &[]] {
                let out = run(args, input);
                assert_eq!(out.status.code(), Some(1), "{input:?}");
                assert_eq!(text(&out.stdout), "");
                assert!(text(&out.stderr).contains("has no canonical form"));
            }
        }
        let out = run(&["--check"], "A { #a\"b }\n");
        assert!(text(&out.stderr).contains("`#a\"b`"));
    }

    #[test]
    fn a_usage_mistake_exits_2() {
        assert_eq!(run(&["--bogus"], "").status.code(), Some(2));
        assert_eq!(run(&["a", "b"], "").status.code(), Some(2));
        assert_eq!(run(&["no/such/file"], "").status.code(), Some(2));
    }
}

mod check {
    use super::{FIXABLE, text};
    use horsegirl_core::check::check;
    use horsegirl_core::diagnostic::LineIndex;
    use std::process::Output;

    fn run(args: &[&str], input: &str) -> Output {
        super::run(&[&["check"], args].concat(), input)
    }

    #[test]
    fn a_clean_file_exits_0_and_says_so() {
        let out = run(&[], "A -> B\n");
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(
            text(&out.stdout),
            "<stdin>: 0 errors, 0 warnings, 0 notes\n"
        );
        assert_eq!(text(&out.stderr), "");
    }

    #[test]
    fn a_warning_alone_exits_0() {
        let out = run(&[], "A -> B\nC\n");
        assert_eq!(out.status.code(), Some(0));
        let stdout = text(&out.stdout);
        assert!(stdout.starts_with("warning[W0107]"), "{stdout}");
        assert!(
            stdout.ends_with("<stdin>: 0 errors, 1 warning, 0 notes\n"),
            "{stdout}"
        );
    }

    #[test]
    fn an_error_exits_1_and_prints_each_diagnostic_with_its_fix() {
        let out = run(&[], FIXABLE);
        assert_eq!(out.status.code(), Some(1));
        let stdout = text(&out.stdout);
        for code in ["E0412", "E0004", "E0300"] {
            assert!(stdout.contains(&format!("error[{code}]")), "{stdout}");
        }
        assert_eq!(stdout.matches("= safe fix:").count(), 3, "{stdout}");
        assert!(stdout.contains("--> <stdin>:4:11"), "{stdout}");
        assert!(stdout.ends_with("\n<stdin>: 3 errors, 0 warnings, 0 notes\n"));
    }

    #[test]
    fn json_prints_one_object_per_line_and_nothing_else() {
        let out = run(&["--json"], FIXABLE);
        assert_eq!(out.status.code(), Some(1));
        let lines = LineIndex::new(FIXABLE);
        let expected: String = check(FIXABLE)
            .iter()
            .map(|d| d.to_json(&lines) + "\n")
            .collect();
        assert_eq!(text(&out.stdout), expected);
        assert_eq!(text(&out.stdout).lines().count(), 3);
    }

    #[test]
    fn json_for_a_clean_file_prints_nothing() {
        let out = run(&["--json"], "A -> B\n");
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(text(&out.stdout), "");
    }

    #[test]
    fn reads_a_file_and_names_it() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/checkout.horse");
        let out = run(&[path], "");
        // The one warning says that stories arrive in v0.2.
        assert_eq!(out.status.code(), Some(0));
        let stdout = text(&out.stdout);
        assert!(stdout.contains(&format!("--> {path}:")), "{stdout}");
        assert!(
            stdout.ends_with(": 0 errors, 1 warning, 0 notes\n"),
            "{stdout}"
        );
    }

    #[test]
    fn a_usage_mistake_exits_2() {
        for args in [
            &["--bogus"][..],
            &["a", "b"],
            &["no/such/file"],
            &["--check"],
        ] {
            let out = run(args, "");
            assert_eq!(out.status.code(), Some(2), "{args:?}");
            assert_eq!(text(&out.stdout), "", "{args:?}");
        }
    }
}

mod fix {
    use super::{FIXABLE, FIXED, scratch, text};
    use std::process::Output;

    fn run(args: &[&str], input: &str) -> Output {
        super::run(&[&["fix"], args].concat(), input)
    }

    #[test]
    fn applies_every_safe_fix_to_standard_input() {
        let out = run(&[], FIXABLE);
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(text(&out.stdout), FIXED);
        assert_eq!(
            text(&out.stderr),
            "<stdin>: 3 safe fixes applied; left: 0 errors, 0 warnings, 0 notes\n"
        );
    }

    #[test]
    fn applies_a_safe_fix_that_appears_after_another() {
        // The N0101 fix overlaps the E0004 fix, so check gives it only after
        // the name is quoted.
        let out = run(&["--json"], "C --> API@v2\n");
        assert_eq!(text(&out.stdout), "C -> \"API@v2\"\n");
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(text(&out.stderr), "");
        let out = run(&[], "C --> API@v2\n");
        assert_eq!(
            text(&out.stderr),
            "<stdin>: 2 safe fixes applied; left: 0 errors, 0 warnings, 0 notes\n"
        );
    }

    #[test]
    fn writes_a_file_in_place() {
        let path = scratch("fix_in_place").join("shop.horse");
        std::fs::write(&path, FIXABLE).unwrap();
        let out = run(&[path.to_str().unwrap()], "");
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(text(&out.stdout), "");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), FIXED);
    }

    #[test]
    fn changes_nothing_that_no_safe_fix_names() {
        // Odd spacing, a comment, a guess and a warning: none has a safe fix.
        let source = "#  a comment\nA->B  ?\n\n  C\nD -> E: pay   later\n";
        let out = run(&[], source);
        assert_eq!(text(&out.stdout), source);
        assert_eq!(out.status.code(), Some(0));
        assert!(
            text(&out.stderr).contains("0 safe fixes applied; left: 0 errors, 1 warning, 1 note")
        );
    }

    #[test]
    fn an_error_with_no_safe_fix_is_left_and_exits_1() {
        // `Carl` is one edit from `Cart` and from `Card`, so it has no fix.
        let source = "A -> Cart\nCart -> B\nC -> Card\nCard -> D\nE -> Carl\n";
        let out = run(&[], source);
        assert_eq!(out.status.code(), Some(1));
        assert_eq!(text(&out.stdout), source);
        let stderr = text(&out.stderr);
        assert!(stderr.starts_with("error[E0412]"), "{stderr}");
        assert!(stderr.ends_with("0 safe fixes applied; left: 1 error, 0 warnings, 0 notes\n"));
    }

    #[test]
    fn json_prints_what_is_left_as_one_object_per_line() {
        let out = run(&["--json"], "A -> B\nA -> -> B\n");
        assert_eq!(out.status.code(), Some(1));
        let stderr = text(&out.stderr);
        assert_eq!(stderr.lines().count(), 1, "{stderr}");
        assert!(stderr.starts_with("{\"code\":\"E0006\""), "{stderr}");
        let out = run(&["--json"], FIXABLE);
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(text(&out.stderr), "");
    }

    #[test]
    fn a_usage_mistake_exits_2_and_touches_no_file() {
        for args in [
            &["--bogus"][..],
            &["a", "b"],
            &["no/such/file"],
            &["-o", "x"],
        ] {
            let out = run(args, FIXABLE);
            assert_eq!(out.status.code(), Some(2), "{args:?}");
            assert_eq!(text(&out.stdout), "", "{args:?}");
        }
    }
}

mod draw {
    use super::{FIXABLE, scratch, text};
    use std::process::Output;

    fn run(args: &[&str], input: &str) -> Output {
        super::run(&[&["draw"], args].concat(), input)
    }

    const SOURCE: &str = "Web app -> API gateway\nAPI gateway -> Orders\nshow flow\n";

    fn is_svg(picture: &str) {
        assert!(
            picture.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""),
            "{picture}"
        );
        assert!(picture.trim_end().ends_with("</svg>"), "{picture}");
    }

    #[test]
    fn writes_the_picture_beside_the_source() {
        let dir = scratch("draw_beside");
        let path = dir.join("shop.horse");
        std::fs::write(&path, SOURCE).unwrap();
        let out = run(&[path.to_str().unwrap()], "");
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(text(&out.stdout), "");
        let svg = dir.join("shop.svg");
        assert_eq!(
            text(&out.stderr),
            format!("horsegirl draw: wrote {}\n", svg.display())
        );
        let picture = std::fs::read_to_string(svg).unwrap();
        is_svg(&picture);
        assert!(picture.contains("<title>shop</title>"));
        assert!(picture.contains(">API gateway<"));
    }

    #[test]
    fn writes_to_the_path_after_o() {
        let dir = scratch("draw_o");
        let out_path = dir.join("picture.svg");
        let out = run(&["--to", "svg", "-o", out_path.to_str().unwrap()], SOURCE);
        assert_eq!(out.status.code(), Some(0));
        is_svg(&std::fs::read_to_string(out_path).unwrap());
    }

    #[test]
    fn prints_the_picture_for_standard_input_or_o_dash() {
        let stdin = run(&[], SOURCE);
        assert_eq!(stdin.status.code(), Some(0));
        is_svg(&text(&stdin.stdout));
        assert!(text(&stdin.stdout).contains("<title>diagram</title>"));
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/checkout.horse");
        let dash = run(&["-o", "-", path], "");
        assert_eq!(dash.status.code(), Some(0));
        is_svg(&text(&dash.stdout));
        // The file's one warning is printed, and it does not stop the picture.
        assert!(text(&dash.stderr).contains("warning[W0010]"));
    }

    #[test]
    fn a_file_with_an_error_gives_no_picture_and_names_each_error() {
        let dir = scratch("draw_error");
        let path = dir.join("shop.horse");
        std::fs::write(&path, FIXABLE).unwrap();
        let out = run(&[path.to_str().unwrap()], "");
        assert_eq!(out.status.code(), Some(1));
        assert_eq!(text(&out.stdout), "");
        assert!(!dir.join("shop.svg").exists());
        let stderr = text(&out.stderr);
        for code in ["E0412", "E0004", "E0300"] {
            assert!(stderr.contains(&format!("error[{code}]")), "{stderr}");
        }
        assert!(
            stderr.contains("has 3 errors, so no picture was drawn"),
            "{stderr}"
        );
        let out = run(&["--json"], FIXABLE);
        assert_eq!(out.status.code(), Some(1));
        assert_eq!(text(&out.stdout), "");
        let stderr = text(&out.stderr);
        let codes: Vec<&str> = stderr
            .lines()
            .map(|line| {
                assert!(
                    line.starts_with("{\"code\":\"") && line.ends_with('}'),
                    "not a JSON object: {line:?}"
                );
                &line[9..14]
            })
            .collect();
        assert_eq!(codes, ["E0412", "E0004", "E0300"], "{stderr}");
    }

    #[test]
    fn a_lens_this_version_does_not_draw_gives_no_picture() {
        let out = run(&[], "A -> B\nshow matrix\n");
        assert_eq!(out.status.code(), Some(1));
        assert_eq!(text(&out.stdout), "");
        let stderr = text(&out.stderr);
        assert!(
            stderr.contains("`show matrix`") && stderr.contains("It draws: flow."),
            "{stderr}"
        );
    }

    #[test]
    fn a_file_with_no_show_line_is_drawn_through_flow() {
        let out = run(&[], "A -> B\n");
        assert_eq!(out.status.code(), Some(0));
        is_svg(&text(&out.stdout));
    }

    #[test]
    fn a_usage_mistake_exits_2_and_writes_no_picture() {
        let dir = scratch("draw_usage");
        let path = dir.join("shop.svg");
        std::fs::write(&path, SOURCE).unwrap();
        let file = path.to_str().unwrap();
        for args in [
            &["--to", "png", "-"][..],
            &["--to"],
            &["-o", "a", "-o", "b"],
            &["--bogus"],
            &["no/such/file"],
            // The picture of shop.svg would overwrite its source.
            &[file],
        ] {
            let out = run(args, SOURCE);
            assert_eq!(out.status.code(), Some(2), "{args:?}");
            assert_eq!(text(&out.stdout), "", "{args:?}");
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), SOURCE);
    }
}
