//! `horsegirl mcp`: the commands `spec`, `check`, `fix` and `draw` as MCP tools
//! over stdio.
//!
//! SPIKE-21 chose the library and the shapes, and `docs/mcp.md` states them. A
//! tool takes the whole text of a file, not a path, so a call has no file
//! system state. A diagnostic is a result, not a failure: `isError` is true
//! only when a tool cannot run.

use std::process::ExitCode;

use horsegirl_core::check::check;
use horsegirl_core::diagnostic::{self, LineIndex};
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::schemars::{self, JsonSchema, Schema, SchemaGenerator, json_schema};
use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router, transport::stdio};
use serde::{Deserialize, Serialize};

const MCP_USAGE: &str = "\
Usage: horsegirl mcp [--config]

Serves the tools spec, check, fix and draw over MCP on standard input and
standard output, until the client closes standard input. An MCP client
starts this command; a person does not run it. docs/mcp.md states the shape
of each tool.

  --config  Print the JSON that adds this server to an MCP client.";

pub fn main(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        None => serve(),
        Some("--config") if args.len() == 1 => config(),
        Some("--help" | "-h") => {
            println!("{MCP_USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("horsegirl mcp: unexpected `{other}`\n\n{MCP_USAGE}");
            ExitCode::from(2)
        }
    }
}

/// Prints the `mcpServers` entry that starts this binary. It names the path
/// of the running binary, so the entry works when `horsegirl` is not on the
/// `PATH` of the client.
fn config() -> ExitCode {
    let path = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("horsegirl mcp: cannot find the path of this binary: {error}");
            return ExitCode::from(2);
        }
    };
    let entry = serde_json::json!({
        "mcpServers": {
            "horsegirl": { "command": path.to_string_lossy(), "args": ["mcp"] }
        }
    });
    match serde_json::to_string_pretty(&entry) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("horsegirl mcp: cannot write the configuration: {error}");
            ExitCode::from(2)
        }
    }
}

fn serve() -> ExitCode {
    // One client talks to one process, so one thread serves it.
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("horsegirl mcp: cannot start: {error}");
            return ExitCode::from(2);
        }
    };
    let served = runtime.block_on(async {
        let service = Horsegirl.serve(stdio()).await?;
        service.waiting().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    });
    match served {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("horsegirl mcp: {error}");
            ExitCode::FAILURE
        }
    }
}

// schemars marks an unsigned integer with a `format` such as "uint", which is
// not a JSON Schema format, and a client's validator warns on it. These state
// the range instead.
fn one_based(_: &mut SchemaGenerator) -> Schema {
    json_schema!({ "type": "integer", "minimum": 1 })
}

fn count(_: &mut SchemaGenerator) -> Schema {
    json_schema!({ "type": "integer", "minimum": 0 })
}

// schemars writes `"type": ["string", "null"]`. A client that maps a schema
// onto one `type` per value rejects that, so each branch names one type.
fn text_or_null(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "anyOf": [{ "type": "string" }, { "type": "null" }],
        "description": "The picture as SVG, or `null` when the file has an error."
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
struct Source {
    /// The whole text of one horsegirl file.
    source: String,
}

/// The `structuredContent` of a `check` result.
#[derive(Debug, Serialize, JsonSchema)]
struct CheckResult {
    /// Every diagnostic, sorted by start position, then by code.
    diagnostics: Vec<Diagnostic>,
    #[schemars(schema_with = "count")]
    errors: usize,
    #[schemars(schema_with = "count")]
    warnings: usize,
    #[schemars(schema_with = "count")]
    notes: usize,
}

impl CheckResult {
    fn of(source: &str, diagnostics: &[diagnostic::Diagnostic]) -> CheckResult {
        let lines = LineIndex::new(source);
        let count = |severity: diagnostic::Severity| {
            let found = diagnostics.iter().filter(|d| d.severity() == severity);
            found.count()
        };
        CheckResult {
            diagnostics: diagnostics
                .iter()
                .map(|d| Diagnostic::of(&lines, d))
                .collect(),
            errors: count(diagnostic::Severity::Error),
            warnings: count(diagnostic::Severity::Warning),
            notes: count(diagnostic::Severity::Note),
        }
    }
}

/// The `structuredContent` of a `fix` result.
#[derive(Debug, Serialize, JsonSchema)]
struct FixResult {
    /// The file with every safe fix made, and no other byte changed.
    source: String,
    /// How many safe fixes were made.
    #[schemars(schema_with = "count")]
    applied: usize,
    /// Every diagnostic that is left in `source`.
    diagnostics: Vec<Diagnostic>,
    #[schemars(schema_with = "count")]
    errors: usize,
    #[schemars(schema_with = "count")]
    warnings: usize,
    #[schemars(schema_with = "count")]
    notes: usize,
}

/// The `structuredContent` of a `draw` result.
#[derive(Debug, Serialize, JsonSchema)]
struct DrawResult {
    #[schemars(schema_with = "text_or_null")]
    svg: Option<String>,
    /// Every diagnostic of the file.
    diagnostics: Vec<Diagnostic>,
    #[schemars(schema_with = "count")]
    errors: usize,
    #[schemars(schema_with = "count")]
    warnings: usize,
    #[schemars(schema_with = "count")]
    notes: usize,
}

// A test holds the bytes of this shape to the `--json` line of the command
// line, as `docs/diagnostics.md` promises.
/// What is wrong in the source, where, and the fix when one exists.
#[derive(Debug, Serialize, JsonSchema)]
struct Diagnostic {
    /// `E`, `W` or `N` and four digits. Stable across releases.
    code: String,
    severity: Severity,
    /// Names the cause and says what to do next.
    message: String,
    span: Span,
    /// The replacement. Always present; `null` when there is none.
    fix: Option<Fix>,
}

impl Diagnostic {
    fn of(lines: &LineIndex<'_>, d: &diagnostic::Diagnostic) -> Diagnostic {
        Diagnostic {
            code: d.code.to_owned(),
            severity: match d.severity() {
                diagnostic::Severity::Error => Severity::Error,
                diagnostic::Severity::Warning => Severity::Warning,
                diagnostic::Severity::Note => Severity::Note,
            },
            message: d.message.clone(),
            span: Span::of(lines, &d.span),
            fix: d.fix.as_ref().map(|fix| Fix {
                confidence: match fix.confidence {
                    diagnostic::Confidence::Safe => Confidence::Safe,
                    diagnostic::Confidence::Suggested => Confidence::Suggested,
                },
                description: fix.description.clone(),
                edits: fix
                    .edits
                    .iter()
                    .map(|edit| Edit {
                        span: Span::of(lines, &edit.span),
                        text: edit.text.clone(),
                    })
                    .collect(),
            }),
        }
    }
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum Severity {
    Error,
    Warning,
    Note,
}

/// A range in the source. `end` is exclusive.
#[derive(Debug, Serialize, JsonSchema)]
struct Span {
    start: Position,
    end: Position,
}

impl Span {
    fn of(lines: &LineIndex<'_>, span: &std::ops::Range<usize>) -> Span {
        let at = |offset| {
            let position = lines.position(offset);
            Position {
                line: position.line,
                column: position.column,
            }
        };
        Span {
            start: at(span.start),
            end: at(span.end),
        }
    }
}

/// Lines and columns start at 1. A column counts Unicode scalar values.
#[derive(Debug, Serialize, JsonSchema)]
struct Position {
    #[schemars(schema_with = "one_based")]
    line: usize,
    #[schemars(schema_with = "one_based")]
    column: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
struct Fix {
    confidence: Confidence,
    /// What the fix does, in one sentence.
    description: String,
    /// Edits against the original source. They do not overlap and are sorted.
    edits: Vec<Edit>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum Confidence {
    /// `fix` applies it with no question.
    Safe,
    /// An agent decides.
    Suggested,
}

#[derive(Debug, Serialize, JsonSchema)]
struct Edit {
    span: Span,
    /// Replaces the text in `span`. An empty text deletes it.
    text: String,
}

#[derive(Clone)]
struct Horsegirl;

#[tool_router]
#[expect(
    clippy::unused_self,
    reason = "rmcp calls each tool as a method; the server holds no state"
)]
impl Horsegirl {
    #[tool(
        description = "Return the language card: the whole horsegirl language on one page. Read it before you write a file."
    )]
    fn spec(&self) -> String {
        horsegirl_core::SPEC.to_owned()
    }

    #[tool(
        description = "Check the text of a horsegirl file. Returns every diagnostic with its code, span, message and fix, and the count of each severity."
    )]
    fn check(&self, Parameters(Source { source }): Parameters<Source>) -> Json<CheckResult> {
        Json(CheckResult::of(&source, &check(&source)))
    }

    #[tool(
        description = "Make every safe fix in the text of a horsegirl file, until none is left. Returns the fixed text, how many fixes were made, and every diagnostic that is left. A suggested fix is left for you to decide."
    )]
    fn fix(&self, Parameters(Source { source }): Parameters<Source>) -> Json<FixResult> {
        let fixed = crate::fix_all(&source);
        let left = CheckResult::of(&fixed.text, &check(&fixed.text));
        Json(FixResult {
            source: fixed.text,
            applied: fixed.applied,
            diagnostics: left.diagnostics,
            errors: left.errors,
            warnings: left.warnings,
            notes: left.notes,
        })
    }

    #[tool(
        description = "Draw the text of a horsegirl file as SVG, through the lens of its first show line, or through flow when it has none. A file with an error gives no picture: svg is null, and the diagnostics say what to fix."
    )]
    fn draw(
        &self,
        Parameters(Source { source }): Parameters<Source>,
    ) -> Result<Json<DrawResult>, String> {
        let checked = CheckResult::of(&source, &check(&source));
        let svg = if checked.errors == 0 {
            Some(crate::picture(&source, "diagram").map_err(|d| d.message)?)
        } else {
            None
        };
        Ok(Json(DrawResult {
            svg,
            diagnostics: checked.diagnostics,
            errors: checked.errors,
            warnings: checked.warnings,
            notes: checked.notes,
        }))
    }
}

// The generated `get_info` answers at once; rmcp's trait makes it async.
#[expect(clippy::unused_async_trait_impl, reason = "the code is generated")]
#[tool_handler(
    name = "horsegirl",
    instructions = "horsegirl is a diagram language. Call spec first and read the card. Then write a file, call check, apply the fixes (fix makes every safe one), and call draw for the SVG."
)]
impl ServerHandler for Horsegirl {}

#[cfg(test)]
mod tests {
    use super::*;

    /// The MCP shape and the `--json` line of the command line are the same
    /// bytes, as `docs/diagnostics.md` promises.
    #[test]
    fn a_diagnostic_is_the_same_bytes_as_its_json_line() {
        let source = "Café --> Payments\nOrders -> Payments\nC -> API@v2\nA {\n";
        let diagnostics = check(source);
        // Both arms of `fix`, and a column past a two-byte letter.
        assert!(
            diagnostics.iter().any(|d| d.fix.is_some()),
            "{diagnostics:?}"
        );
        assert!(
            diagnostics.iter().any(|d| d.fix.is_none()),
            "{diagnostics:?}"
        );
        let lines = LineIndex::new(source);
        for d in &diagnostics {
            let value = serde_json::to_value(Diagnostic::of(&lines, d)).unwrap();
            assert_eq!(serde_json::to_string(&value).unwrap(), d.to_json(&lines));
        }
    }
}
