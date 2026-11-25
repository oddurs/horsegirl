use std::path::{Path, PathBuf};
use std::process::ExitCode;

use horsegirl_bench::{Command, Recorded, Recording, Task, load_tasks, prompt, report, run};

const USAGE: &str = "\
horsegirl-bench: can a model write horsegirl from the language card?

Usage:
  horsegirl-bench prompt [--tasks DIR] ID
      Print what a model reads for task ID: the card, the instruction and
      the description of the system.
  horsegirl-bench score [--tasks DIR] [--name NAME] REPLIES
      Score the replies recorded in REPLIES, one file <id>.txt for each
      task, and print the report. Needs no network.
  horsegirl-bench run [--tasks DIR] [--name NAME] --command CMD REPLIES
      Run CMD through `sh -c` once for each task, with the prompt on standard
      input and HORSEGIRL_TASK set to the id. Record each reply in REPLIES as
      <id>.txt, then print the report.

DIR is bench/tasks when it is not given. NAME is the name of the report, and
the last part of REPLIES when it is not given. Exit codes: 0 when the report
is printed, 1 when a file or the model fails, 2 for a mistake in the command
line.";

/// What the command line asks for.
enum Action {
    Prompt { id: String },
    Score { replies: PathBuf },
    Run { command: String, replies: PathBuf },
}

/// The command line, read.
struct Args {
    action: Action,
    tasks: PathBuf,
    name: Option<String>,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let args = match read(&args) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("horsegirl-bench: {message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let tasks = match load_tasks(&args.tasks) {
        Ok(tasks) => tasks,
        Err(error) => {
            eprintln!(
                "horsegirl-bench: cannot read the tasks in {}: {error}",
                args.tasks.display()
            );
            return ExitCode::FAILURE;
        }
    };
    match args.action {
        Action::Prompt { id } => print_prompt(&tasks, &id),
        Action::Score { replies } => {
            // A mistyped path would otherwise score as a run with no replies.
            if !replies.is_dir() {
                eprintln!(
                    "horsegirl-bench: {} is not a directory. Give the directory that holds the replies, one <id>.txt for each task.",
                    replies.display()
                );
                return ExitCode::FAILURE;
            }
            let name = args.name.unwrap_or_else(|| name_of(&replies));
            let scores = run(&tasks, &mut Recorded { dir: replies });
            print_report(&name, scores)
        }
        Action::Run { command, replies } => {
            let name = args.name.unwrap_or_else(|| name_of(&replies));
            let mut model = Command { command };
            let mut recording = Recording {
                model: &mut model,
                dir: replies,
            };
            print_report(&name, run(&tasks, &mut recording))
        }
    }
}

fn read(args: &[String]) -> Result<Args, String> {
    let mut args = args.iter();
    let command = args.next().map_or("", String::as_str);
    if !matches!(command, "prompt" | "score" | "run") {
        return Err(format!("unknown command `{command}`"));
    }
    let mut tasks = PathBuf::from("bench/tasks");
    let mut name = None;
    let mut model = None;
    let mut operand: Option<String> = None;
    while let Some(arg) = args.next() {
        let mut value = || {
            args.next()
                .cloned()
                .ok_or_else(|| format!("`{arg}` needs a value"))
        };
        match arg.as_str() {
            "--tasks" => tasks = PathBuf::from(value()?),
            "--name" if command != "prompt" => name = Some(value()?),
            "--command" if command == "run" => model = Some(value()?),
            option if option.starts_with('-') => {
                return Err(format!("unknown option `{option}`"));
            }
            _ if operand.is_some() => return Err(format!("unexpected `{arg}`")),
            _ => operand = Some(arg.clone()),
        }
    }
    let action = match (command, operand, model) {
        ("prompt", Some(id), _) => Action::Prompt { id },
        ("prompt", None, _) => return Err("`prompt` needs a task id".to_owned()),
        ("score", Some(replies), _) => Action::Score {
            replies: PathBuf::from(replies),
        },
        ("run", Some(replies), Some(command)) => Action::Run {
            command,
            replies: PathBuf::from(replies),
        },
        ("run", _, None) => return Err("`run` needs --command".to_owned()),
        _ => return Err(format!("`{command}` needs the directory of the replies")),
    };
    Ok(Args {
        action,
        tasks,
        name,
    })
}

fn print_prompt(tasks: &[Task], id: &str) -> ExitCode {
    let Some(task) = tasks.iter().find(|task| task.id == id) else {
        eprintln!(
            "horsegirl-bench: no task has the id `{id}`. An id is the three digits at the start of a task file name."
        );
        return ExitCode::FAILURE;
    };
    print!("{}", prompt(task));
    ExitCode::SUCCESS
}

fn print_report(name: &str, scores: std::io::Result<Vec<horsegirl_bench::Score>>) -> ExitCode {
    match scores {
        Ok(scores) => {
            print!("{}", report::render(name, &scores));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("horsegirl-bench: {error}");
            ExitCode::FAILURE
        }
    }
}

fn name_of(dir: &Path) -> String {
    dir.file_name().map_or_else(
        || dir.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}
