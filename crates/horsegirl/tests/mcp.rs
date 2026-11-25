//! A scripted MCP client: it starts `horsegirl mcp`, speaks JSON-RPC on its
//! standard input and reads the answers from its standard output. Each tool
//! result is held to the shape in `docs/mcp.md`, and to the command line that
//! does the same work.

use std::io::Write;
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

fn horsegirl(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_horsegirl"))
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

/// Sends a legacy `initialize`, then each request in `calls` with the ids 1,
/// 2 and so on, and closes standard input. Returns the answer to each request,
/// in the order of `calls`.
fn session(calls: &[Value]) -> Vec<Value> {
    let mut input = String::new();
    let mut push = |message: Value| {
        input.push_str(&message.to_string());
        input.push('\n');
    };
    push(json!({
        "jsonrpc": "2.0", "id": 0, "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": { "name": "scripted", "version": "0" }
        }
    }));
    push(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
    for (id, call) in (1..).zip(calls) {
        let mut request = json!({ "jsonrpc": "2.0", "id": id });
        let fields = call.as_object().expect("a call is an object");
        for (key, value) in fields {
            request[key] = value.clone();
        }
        push(request);
    }
    let out = horsegirl(&["mcp"], &input);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let answers: Vec<Value> = text(&out.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("each line is JSON"))
        .collect();
    let by_id = |id: usize| {
        let found = answers.iter().find(|answer| answer["id"] == json!(id));
        found
            .cloned()
            .unwrap_or_else(|| panic!("no answer to {id}: {answers:?}"))
    };
    let initialize = by_id(0);
    assert_eq!(initialize["result"]["serverInfo"]["name"], "horsegirl");
    (1..=calls.len()).map(by_id).collect()
}

fn call(tool: &str, arguments: &Value) -> Value {
    json!({ "method": "tools/call", "params": { "name": tool, "arguments": arguments } })
}

/// The result of a tool that returns JSON: `isError` is false, and the one
/// text block holds the same JSON as `structuredContent`.
fn structured(answer: &Value) -> Value {
    let result = &answer["result"];
    assert_eq!(result["isError"], false, "{answer}");
    let content = result["content"].as_array().expect("content is an array");
    assert_eq!(content.len(), 1, "{answer}");
    assert_eq!(content[0]["type"], "text");
    let block: Value = serde_json::from_str(content[0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(block, result["structuredContent"]);
    result["structuredContent"].clone()
}

/// What `horsegirl check --json` prints for `source`, as one array.
fn cli_diagnostics(source: &str) -> Value {
    let out = horsegirl(&["check", "--json"], source);
    let lines = text(&out.stdout);
    let parsed = lines
        .lines()
        .map(|line| serde_json::from_str(line).unwrap());
    Value::Array(parsed.collect())
}

/// Three errors with a safe fix, one with none, two warnings and a note with
/// a safe fix. The note sits past a two-byte letter, so a column that counts
/// bytes shows.
const MIXED: &str =
    "Café --> Payments\nPayments -> Bank\nOrders -> Paymnts\nC -> API@v2\nshow flwo\nAlone\nA {\n";

#[test]
fn the_server_lists_four_tools_and_their_schemas() {
    let answers = session(&[json!({ "method": "tools/list" })]);
    let tools = answers[0]["result"]["tools"].as_array().unwrap().clone();
    let mut names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    names.sort_unstable();
    assert_eq!(names, ["check", "draw", "fix", "spec"]);
    for tool in &tools {
        let name = tool["name"].as_str().unwrap();
        assert!(tool["description"].as_str().is_some_and(|d| !d.is_empty()));
        assert_eq!(tool["inputSchema"]["type"], "object", "{name}");
        if name == "spec" {
            assert!(tool.get("outputSchema").is_none(), "{tool}");
        } else {
            assert_eq!(tool["inputSchema"]["required"], json!(["source"]), "{name}");
            assert_eq!(tool["outputSchema"]["type"], "object", "{name}");
        }
    }
    // schemars writes formats such as "uint" that no validator knows.
    let list = answers[0]["result"].to_string();
    assert!(!list.contains("\"format\""), "{list}");
    // A client that maps a schema onto one type per value rejects a list.
    assert!(!list.contains("\"type\":["), "{list}");
}

#[test]
fn spec_returns_the_card_as_text() {
    let answers = session(&[call("spec", &json!({}))]);
    let result = &answers[0]["result"];
    assert_eq!(result["isError"], false);
    assert_eq!(result["content"][0]["text"], horsegirl_core::SPEC);
}

#[test]
fn check_returns_every_diagnostic_with_its_fix_in_one_call() {
    let answers = session(&[call("check", &json!({ "source": MIXED }))]);
    let result = structured(&answers[0]);
    let diagnostics = cli_diagnostics(MIXED);
    let all = diagnostics.as_array().unwrap();
    let severity = |word: &str| all.iter().filter(|d| d["severity"] == word).count();
    assert_eq!(
        result,
        json!({
            "diagnostics": diagnostics,
            "errors": severity("error"),
            "warnings": severity("warning"),
            "notes": severity("note"),
        })
    );
    // Every diagnostic arrives in this one answer, with its fix or `null`.
    assert_eq!(all.len(), 7, "{result}");
    let fixes = all.iter().filter(|d| d["fix"]["confidence"] == "safe");
    assert_eq!(fixes.count(), 4, "{result}");
    assert_eq!(
        (severity("error"), severity("warning"), severity("note")),
        (4, 2, 1)
    );
}

#[test]
fn check_of_a_clean_file_returns_no_diagnostic() {
    let answers = session(&[call("check", &json!({ "source": "A -> B\n" }))]);
    assert_eq!(
        structured(&answers[0]),
        json!({ "diagnostics": [], "errors": 0, "warnings": 0, "notes": 0 })
    );
}

#[test]
fn fix_returns_what_the_command_line_writes_and_what_is_left() {
    let answers = session(&[call("fix", &json!({ "source": MIXED }))]);
    let result = structured(&answers[0]);
    let out = horsegirl(&["fix", "--json"], MIXED);
    let fixed = text(&out.stdout);
    assert_ne!(fixed, MIXED);
    assert_eq!(result["source"], fixed.as_str());
    assert_eq!(result["applied"], 4, "{result}");
    let left = cli_diagnostics(&fixed);
    assert_eq!(result["diagnostics"], left);
    let left = left.as_array().unwrap();
    let errors = left.iter().filter(|d| d["severity"] == "error").count();
    assert_eq!(result["errors"], errors);
}

#[test]
fn draw_returns_the_svg_that_the_command_line_prints() {
    let source = "Web app -> API gateway\nAPI gateway -> Orders\nshow flow\n";
    let answers = session(&[call("draw", &json!({ "source": source }))]);
    let result = structured(&answers[0]);
    let picture = text(&horsegirl(&["draw"], source).stdout);
    assert!(picture.starts_with("<svg"), "{picture}");
    assert_eq!(result["svg"], picture.as_str());
    assert_eq!(result["errors"], 0);
}

#[test]
fn draw_of_a_file_with_an_error_returns_no_svg_and_the_diagnostics() {
    let answers = session(&[call("draw", &json!({ "source": MIXED }))]);
    let result = structured(&answers[0]);
    assert_eq!(result["svg"], Value::Null);
    assert_eq!(result["diagnostics"], cli_diagnostics(MIXED));
}

#[test]
fn a_tool_that_cannot_run_returns_is_error_with_the_reason() {
    let answers = session(&[
        call("draw", &json!({ "source": "A -> B\nshow matrix\n" })),
        call("check", &json!({})),
        call("render", &json!({ "source": "A -> B\n" })),
    ]);
    let lens = &answers[0]["result"];
    assert_eq!(lens["isError"], true, "{lens}");
    let reason = lens["content"][0]["text"].as_str().unwrap();
    assert!(reason.contains("`show matrix`") && reason.contains("It draws: flow."));
    let missing = &answers[1]["result"];
    assert_eq!(missing["isError"], true, "{missing}");
    assert!(
        missing["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("source")
    );
    assert_eq!(answers[2]["error"]["code"], -32602, "{}", answers[2]);
}

#[test]
fn config_prints_an_entry_that_starts_this_binary() {
    let out = horsegirl(&["mcp", "--config"], "");
    assert_eq!(out.status.code(), Some(0));
    let config: Value = serde_json::from_slice(&out.stdout).unwrap();
    let server = &config["mcpServers"]["horsegirl"];
    assert_eq!(server["args"], json!(["mcp"]));
    let command = server["command"].as_str().unwrap();
    let binary = std::fs::canonicalize(env!("CARGO_BIN_EXE_horsegirl")).unwrap();
    assert_eq!(std::fs::canonicalize(command).unwrap(), binary);
}

#[test]
fn an_unexpected_argument_exits_2() {
    let out = horsegirl(&["mcp", "--port"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(text(&out.stderr).contains("`--port`"));
}
