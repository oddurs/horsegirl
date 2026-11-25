//! SPIKE-21 prototype: a stdio MCP server with one tool, `check`.
//!
//! The check itself is a stand-in, because the parser does not exist yet. It
//! reports each `-->` as a mistyped arrow and offers `->` as a safe fix. That
//! is enough to put a real span and a real fix on the wire, which is what the
//! spike must fix the shape of.

use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::schemars::{self, Schema, SchemaGenerator, json_schema};
use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router, transport::stdio};
use serde::{Deserialize, Serialize};

// schemars marks a `u32` with `"format": "uint32"`, which is not a JSON Schema
// format. A client's validator warns on it, so the shape states the range.
fn one_based(_: &mut SchemaGenerator) -> Schema {
    json_schema!({ "type": "integer", "minimum": 1 })
}

fn count(_: &mut SchemaGenerator) -> Schema {
    json_schema!({ "type": "integer", "minimum": 0 })
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CheckParams {
    /// The whole text of one horsegirl file.
    source: String,
}

/// The `structuredContent` of a `check` result.
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct CheckResult {
    /// Every diagnostic, sorted by start position, then by code.
    diagnostics: Vec<Diagnostic>,
    #[schemars(schema_with = "count")]
    errors: u32,
    #[schemars(schema_with = "count")]
    warnings: u32,
    #[schemars(schema_with = "count")]
    notes: u32,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct Diagnostic {
    /// `E`, `W` or `N` and four digits. Stable across releases.
    code: String,
    severity: Severity,
    /// Names the cause and says what to do next.
    message: String,
    span: Span,
    /// The replacement, when one exists. Always present; `null` when none.
    fix: Option<Fix>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
#[expect(
    dead_code,
    reason = "the prototype emits errors only; the shape names all three"
)]
enum Severity {
    Error,
    Warning,
    Note,
}

/// A range in the source. `end` is exclusive.
#[derive(Debug, Clone, Copy, Serialize, schemars::JsonSchema)]
struct Span {
    start: Position,
    end: Position,
}

/// Lines and columns start at 1. A column counts Unicode scalar values.
#[derive(Debug, Clone, Copy, Serialize, schemars::JsonSchema)]
struct Position {
    #[schemars(schema_with = "one_based")]
    line: u32,
    #[schemars(schema_with = "one_based")]
    column: u32,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct Fix {
    confidence: Confidence,
    /// What the fix does, in one sentence.
    description: String,
    /// Edits against the original source. They do not overlap and are sorted.
    edits: Vec<Edit>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
#[expect(
    dead_code,
    reason = "the prototype emits safe fixes only; the shape names both"
)]
enum Confidence {
    /// `fix` applies it with no question.
    Safe,
    /// An agent must decide.
    Suggested,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct Edit {
    span: Span,
    /// Replaces the text in `span`. Empty deletes it.
    text: String,
}

fn check(source: &str) -> CheckResult {
    let mut diagnostics = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let line_no = u32::try_from(index + 1).expect("a file has fewer than 2^32 lines");
        for (byte, _) in line.match_indices("-->") {
            let column = column_at(line, byte);
            let span = Span {
                start: Position {
                    line: line_no,
                    column,
                },
                end: Position {
                    line: line_no,
                    column: column + 3,
                },
            };
            diagnostics.push(Diagnostic {
                code: "E0001".to_owned(),
                severity: Severity::Error,
                message: "`-->` is not an arrow. Write `->` for a link.".to_owned(),
                span,
                fix: Some(Fix {
                    confidence: Confidence::Safe,
                    description: "Replace `-->` with `->`.".to_owned(),
                    edits: vec![Edit {
                        span,
                        text: "->".to_owned(),
                    }],
                }),
            });
        }
    }
    let errors = u32::try_from(diagnostics.len()).expect("fewer than 2^32 diagnostics");
    CheckResult {
        diagnostics,
        errors,
        warnings: 0,
        notes: 0,
    }
}

fn column_at(line: &str, byte: usize) -> u32 {
    u32::try_from(line[..byte].chars().count() + 1).expect("a line has fewer than 2^32 chars")
}

#[derive(Clone)]
struct Horsegirl;

#[tool_router]
impl Horsegirl {
    #[tool(
        description = "Check a horsegirl file. Returns every diagnostic with its code, span and fix."
    )]
    fn check(
        &self,
        Parameters(CheckParams { source }): Parameters<CheckParams>,
    ) -> Json<CheckResult> {
        Json(check(&source))
    }
}

#[tool_handler(name = "horsegirl", version = "0.0.0")]
impl ServerHandler for Horsegirl {}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = Horsegirl.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mistyped_arrow_gives_one_diagnostic_with_a_safe_fix() {
        // rmcp sends a `serde_json::Value`, whose keys are sorted by name, so
        // the golden file goes through the same conversion.
        let value = serde_json::to_value(check("Café --> Payments\n")).unwrap();
        let json = serde_json::to_string_pretty(&value).unwrap();
        assert_eq!(json, include_str!("../check.golden.json").trim_end());
    }
}
