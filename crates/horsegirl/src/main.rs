use std::io::Read;
use std::process::ExitCode;

const USAGE: &str = "\
horsegirl: an agent-first diagram language

Usage: horsegirl <command>

Commands:
  spec       Print the one-page language card
  fmt        Print a file in its one canonical form
  --version  Print the version
  --help     Print this help";

const FMT_USAGE: &str = "\
Usage: horsegirl fmt [--check] [FILE]

Prints FILE, or standard input when FILE is `-` or absent, in its canonical
form. With --check, prints nothing and exits 1 when the file would change.
A file with an error has no canonical form: fmt names each error and exits 1.";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("spec") => {
            print!("{}", horsegirl_core::SPEC);
            ExitCode::SUCCESS
        }
        Some("fmt") => fmt(args.collect()),
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

fn fmt(args: Vec<String>) -> ExitCode {
    let mut check = false;
    let mut file = None;
    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            "--help" | "-h" => {
                println!("{FMT_USAGE}");
                return ExitCode::SUCCESS;
            }
            flag if flag.starts_with('-') && flag != "-" => {
                eprintln!("horsegirl fmt: unknown option `{flag}`\n\n{FMT_USAGE}");
                return ExitCode::from(2);
            }
            _ if file.is_some() => {
                eprintln!("horsegirl fmt: one file at a time\n\n{FMT_USAGE}");
                return ExitCode::from(2);
            }
            _ => file = Some(arg),
        }
    }
    let name = file.as_deref().filter(|name| *name != "-");
    let read = if let Some(path) = name {
        std::fs::read_to_string(path)
    } else {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text).map(|_| text)
    };
    let label = name.unwrap_or("<stdin>");
    let source = match read {
        Ok(source) => source,
        Err(error) => {
            eprintln!("horsegirl fmt: cannot read {label}: {error}");
            return ExitCode::from(2);
        }
    };
    match horsegirl_core::format::format(&source) {
        Ok(text) if check => {
            if text == source {
                ExitCode::SUCCESS
            } else {
                eprintln!("horsegirl fmt: {label} would change");
                ExitCode::FAILURE
            }
        }
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                let line = source[..error.span.start].matches('\n').count() + 1;
                eprintln!("{label}:{line}: {} {}", error.code, error.message);
            }
            ExitCode::FAILURE
        }
    }
}
