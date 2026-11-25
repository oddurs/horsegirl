use std::process::Command;

fn horsegirl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_horsegirl"))
}

#[test]
fn spec_prints_the_card() {
    let out = horsegirl().arg("spec").output().expect("the binary runs");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), horsegirl_core::SPEC);
}

#[test]
fn an_unknown_command_exits_2_and_names_it() {
    let out = horsegirl().arg("gallop").output().expect("the binary runs");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("`gallop`"));
}

mod fmt {
    use super::horsegirl;
    use std::io::Write;
    use std::process::{Output, Stdio};

    /// Runs `horsegirl fmt` with `args` and `input` on standard input.
    fn run(args: &[&str], input: &str) -> Output {
        let mut child = horsegirl()
            .arg("fmt")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the binary runs");
        child
            .stdin
            .take()
            .expect("stdin is piped")
            .write_all(input.as_bytes())
            .expect("the input is written");
        child.wait_with_output().expect("the binary ends")
    }

    fn text(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).into_owned()
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
        assert!(text(&out.stderr).starts_with("<stdin>:2: E0006"));
    }

    #[test]
    fn a_usage_mistake_exits_2() {
        assert_eq!(run(&["--bogus"], "").status.code(), Some(2));
        assert_eq!(run(&["a", "b"], "").status.code(), Some(2));
        assert_eq!(run(&["no/such/file"], "").status.code(), Some(2));
    }
}
