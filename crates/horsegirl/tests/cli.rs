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
