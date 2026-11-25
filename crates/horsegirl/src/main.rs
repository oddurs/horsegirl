use std::process::ExitCode;

const USAGE: &str = "\
horsegirl: an agent-first diagram language

Usage: horsegirl <command>

Commands:
  spec       Print the one-page language card
  --version  Print the version
  --help     Print this help";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("spec") => {
            print!("{}", horsegirl_core::SPEC);
            ExitCode::SUCCESS
        }
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
