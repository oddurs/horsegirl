//! The benchmark: can a model write horsegirl from the language card alone?
//!
//! A task is a description of a system and a checker. The runner gives a
//! model the card and the description, takes the file it writes, and scores
//! three things: whether the file is valid on the first try, whether it is
//! valid after one `fix`, and whether it states every fact that the checker
//! expects. A model sits behind the [`Model`] trait, so the runner never
//! knows how a reply was made, and a recorded reply scores with no network.

pub mod adapter;
pub mod checker;
pub mod report;

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use horsegirl_core::check::{check, fix_all};
use horsegirl_core::diagnostic::Severity;
use horsegirl_core::model::lower;
use horsegirl_core::syntax::parse;

pub use adapter::{Command, Model, Recorded, Recording};

/// The extension of a task file.
pub const TASK_EXTENSION: &str = "task";

/// The line that ends the description of a task and starts its checker.
const SEPARATOR: &str = "---";

/// One task: a system to describe and the facts a correct file states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    /// The three digits at the start of the file name.
    pub id: String,
    pub title: String,
    /// What the model reads after the card.
    pub description: String,
    /// A horsegirl file that states every fact a correct answer must state.
    pub checker: String,
}

/// A task file that cannot be read as a task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskError {
    pub file: String,
    pub message: String,
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.file, self.message)
    }
}

impl std::error::Error for TaskError {}

impl Task {
    /// Reads a task from the text of its file.
    ///
    /// The first line is the title. The description follows, then a line
    /// that is `---` alone, then the checker.
    ///
    /// # Errors
    ///
    /// The file name does not start with three digits and a `-`, the file has
    /// no title, no description or no `---` line, or the checker is empty.
    pub fn parse(file_name: &str, text: &str) -> Result<Task, TaskError> {
        let error = |message: &str| TaskError {
            file: file_name.to_owned(),
            message: message.to_owned(),
        };
        let id = file_name.get(..3).filter(|id| {
            id.bytes().all(|b| b.is_ascii_digit()) && file_name.as_bytes().get(3) == Some(&b'-')
        });
        let Some(id) = id else {
            return Err(error(
                "the file name must start with three digits and a `-`, as in `001-checkout.task`",
            ));
        };
        let mut lines = text.lines();
        let title = lines.next().unwrap_or("").trim();
        if title.is_empty() {
            return Err(error("the first line must be the title"));
        }
        let rest: Vec<&str> = lines.collect();
        let Some(split) = rest.iter().position(|line| line.trim_end() == SEPARATOR) else {
            return Err(error(
                "a line that is `---` alone must end the description and start the checker",
            ));
        };
        let description = rest[..split].join("\n").trim().to_owned();
        let checker = rest[split + 1..].join("\n").trim().to_owned() + "\n";
        if description.is_empty() {
            return Err(error("the description is empty"));
        }
        if checker.trim().is_empty() {
            return Err(error("the checker is empty"));
        }
        Ok(Task {
            id: id.to_owned(),
            title: title.to_owned(),
            description,
            checker,
        })
    }
}

/// Every task in `dir`, in the order of the file names.
///
/// # Errors
///
/// The directory cannot be read, or a task file cannot be read as a task.
pub fn load_tasks(dir: &Path) -> io::Result<Vec<Task>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == TASK_EXTENSION) {
            paths.push(path);
        }
    }
    // read_dir has no order, and the report must be the same on every machine.
    paths.sort();
    let mut tasks = Vec::with_capacity(paths.len());
    for path in paths {
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let text = fs::read_to_string(&path)?;
        let task = Task::parse(&name, &text)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if tasks.iter().any(|known: &Task| known.id == task.id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{name}: another task has the id {}", task.id),
            ));
        }
        tasks.push(task);
    }
    Ok(tasks)
}

/// What a model reads for `task`: the card, the instruction, the description.
#[must_use]
pub fn prompt(task: &Task) -> String {
    format!(
        "{card}\nWrite a horsegirl file for the system below, using only the card above. \
Write each name and each label in backticks exactly as it is written, without the backticks. \
Reply with the file alone, in one fenced code block.\n\n{description}\n",
        card = horsegirl_core::SPEC,
        description = task.description
    )
}

/// The file in a reply: the first fenced code block, or the whole reply when
/// it has no fence. A fence that never closes runs to the end of the reply.
#[must_use]
pub fn extract(reply: &str) -> String {
    let mut lines = reply.lines();
    if !lines.any(|line| line.trim_start().starts_with("```")) {
        return reply.to_owned();
    }
    let mut file = String::new();
    for line in lines {
        if line.trim_start().starts_with("```") {
            break;
        }
        file.push_str(line);
        file.push('\n');
    }
    file
}

/// What one reply scored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Score {
    pub id: String,
    pub title: String,
    /// `None` when the model gave no reply for the task.
    pub answer: Option<Answer>,
}

/// The score of a reply that exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    /// The code of each diagnostic of the first try, in the order `check`
    /// gives them.
    pub first: Vec<&'static str>,
    /// `check` gives no error, and the file states a link or a rule.
    pub valid_first: bool,
    /// The code of each diagnostic left after one `fix`.
    pub left: Vec<&'static str>,
    pub valid_fixed: bool,
    /// How many facts the checker expects.
    pub expected: usize,
    /// Each expected fact that the fixed file does not state.
    pub missing: Vec<String>,
}

impl Answer {
    /// Valid after one `fix`, and every expected fact stated.
    #[must_use]
    pub fn correct(&self) -> bool {
        self.valid_fixed && self.missing.is_empty()
    }
}

/// Whether `file` is a valid answer: `check` gives it no error, and it
/// states at least one link or `-x` rule.
///
/// An empty reply, an empty fenced block and a refusal in prose all give no
/// error, because prose reads as one lone thing. None of them describes a
/// system, so none of them counts as valid.
fn valid(file: &str, codes: &[&str]) -> bool {
    let model = lower(&parse(file).tree);
    let states = !model.links.is_empty() || !model.rules.is_empty();
    states
        && !codes
            .iter()
            .any(|code| Severity::of(code) == Some(Severity::Error))
}

/// Scores one reply to `task`.
#[must_use]
pub fn score(task: &Task, reply: Option<&str>) -> Score {
    let answer = reply.map(|reply| {
        let file = extract(reply);
        let first: Vec<&'static str> = check(&file).iter().map(|d| d.code).collect();
        let fixed = fix_all(&file).text;
        let left: Vec<&'static str> = check(&fixed).iter().map(|d| d.code).collect();
        let expectations = checker::expectations(&task.checker);
        let missing = checker::missing(&expectations, &fixed);
        Answer {
            valid_first: valid(&file, &first),
            first,
            valid_fixed: valid(&fixed, &left),
            left,
            expected: expectations.len(),
            missing,
        }
    });
    Score {
        id: task.id.clone(),
        title: task.title.clone(),
        answer,
    }
}

/// Asks `model` for a file for each task, and scores each reply.
///
/// # Errors
///
/// The model fails for a reason other than having no reply.
pub fn run(tasks: &[Task], model: &mut dyn Model) -> io::Result<Vec<Score>> {
    let mut scores = Vec::with_capacity(tasks.len());
    for task in tasks {
        let reply = model.reply(task, &prompt(task))?;
        scores.push(score(task, reply.as_deref()));
    }
    Ok(scores)
}
