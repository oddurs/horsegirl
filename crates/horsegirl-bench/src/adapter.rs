//! The adapter between the runner and a model.
//!
//! The runner knows a model only as [`Model`]: a prompt goes in, a reply comes
//! out. [`Recorded`] reads replies from files, so a scored run needs no
//! network. [`Command`] runs any program that reads a prompt on standard
//! input and writes a reply on standard output, so a model of any vendor
//! joins the benchmark with a short script and no change to this crate.
//! [`Recording`] keeps the replies of another adapter as files that
//! [`Recorded`] reads.

use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::Task;

/// The extension of a recorded reply.
pub const REPLY_EXTENSION: &str = "txt";

/// A model that writes a reply to a prompt.
pub trait Model {
    /// The reply to `prompt`, which asks for a file for `task`, or `None`
    /// when the model has no reply for the task.
    ///
    /// # Errors
    ///
    /// The model cannot be reached or fails.
    fn reply(&mut self, task: &Task, prompt: &str) -> io::Result<Option<String>>;
}

/// The path of the recorded reply to `task` in `dir`.
#[must_use]
pub fn reply_path(dir: &Path, task: &Task) -> PathBuf {
    dir.join(format!("{}.{REPLY_EXTENSION}", task.id))
}

/// Replies read from a directory: `<id>.txt` for each task. A task with no
/// file has no reply.
pub struct Recorded {
    pub dir: PathBuf,
}

impl Model for Recorded {
    fn reply(&mut self, task: &Task, _prompt: &str) -> io::Result<Option<String>> {
        match fs::read_to_string(reply_path(&self.dir, task)) {
            Ok(reply) => Ok(Some(reply)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }
}

/// A program run once for each task through `sh -c`. It reads the prompt on
/// standard input and writes the reply on standard output. The variable
/// `HORSEGIRL_TASK` holds the id of the task.
pub struct Command {
    pub command: String,
}

impl Model for Command {
    fn reply(&mut self, task: &Task, prompt: &str) -> io::Result<Option<String>> {
        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .env("HORSEGIRL_TASK", &task.id)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("the standard input of the model is not a pipe"))?;
        let written = stdin.write_all(prompt.as_bytes());
        // Closing the pipe tells the program that the prompt is complete.
        drop(stdin);
        // A program may answer without reading the prompt and close the pipe.
        match written {
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {}
            written => written?,
        }
        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(io::Error::other(format!(
                "the model command failed for task {} with {}",
                task.id, output.status
            )));
        }
        String::from_utf8(output.stdout)
            .map(Some)
            .map_err(|_| io::Error::other(format!("the reply to task {} is not UTF-8", task.id)))
    }
}

/// Another adapter, with each reply it gives written to `dir` as `<id>.txt`.
pub struct Recording<'m> {
    pub model: &'m mut dyn Model,
    pub dir: PathBuf,
}

impl Model for Recording<'_> {
    fn reply(&mut self, task: &Task, prompt: &str) -> io::Result<Option<String>> {
        let reply = self.model.reply(task, prompt)?;
        if let Some(reply) = &reply {
            fs::create_dir_all(&self.dir)?;
            fs::write(reply_path(&self.dir, task), reply)?;
        }
        Ok(reply)
    }
}
