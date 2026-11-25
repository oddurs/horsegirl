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

/// The two eras of the protocol that the server serves.
#[derive(Clone, Copy, Debug)]
enum Era {
    /// `2025-11-25`: `initialize` opens a session, and a request carries no
    /// protocol state.
    Legacy,
    /// `2026-07-28`: `server/discover` opens, and every request carries the
    /// protocol version and the client capabilities in `_meta`.
    Modern,
}

/// The `_meta` that a `2026-07-28` client puts on each request.
fn modern_meta() -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientCapabilities": {},
        "io.modelcontextprotocol/clientInfo": { "name": "scripted", "version": "0" }
    })
}

/// Sends a legacy `initialize`, then each request in `calls` with the ids 1,
/// 2 and so on, and closes standard input. Returns the answer to each request,
/// in the order of `calls`.
fn session(calls: &[Value]) -> Vec<Value> {
    session_in(Era::Legacy, calls)
}

/// As [`session`], but opens the session the way a client of `era` does.
fn session_in(era: Era, calls: &[Value]) -> Vec<Value> {
    let mut input = String::new();
    let mut push = |message: Value| {
        input.push_str(&message.to_string());
        input.push('\n');
    };
    match era {
        Era::Legacy => {
            push(json!({
                "jsonrpc": "2.0", "id": 0, "method": "initialize",
                "params": {
                    "protocolVersion": "2025-11-25",
                    "capabilities": {},
                    "clientInfo": { "name": "scripted", "version": "0" }
                }
            }));
            push(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        }
        Era::Modern => push(json!({
            "jsonrpc": "2.0", "id": 0, "method": "server/discover",
            "params": { "_meta": modern_meta() }
        })),
    }
    for (id, call) in (1..).zip(calls) {
        let mut request = json!({ "jsonrpc": "2.0", "id": id });
        let fields = call.as_object().expect("a call is an object");
        for (key, value) in fields {
            request[key] = value.clone();
        }
        if let Era::Modern = era {
            if request.get("params").is_none() {
                request["params"] = json!({});
            }
            request["params"]["_meta"] = modern_meta();
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
    let opened = by_id(0);
    let server_info = match era {
        Era::Legacy => &opened["result"]["serverInfo"],
        Era::Modern => &opened["result"]["_meta"]["io.modelcontextprotocol/serverInfo"],
    };
    assert_eq!(server_info["name"], "horsegirl", "{opened}");
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
    // A client that maps a schema onto one type per value rejects a list.
    let list = answers[0]["result"].to_string();
    assert!(!list.contains("\"type\":["), "{list}");
}

/// The committed tool list. The schemas are derived from Rust types, so a
/// renamed field changes the contract without a visible diff in the source.
/// This file makes the change visible in review.
const TOOLS: &str = "tests/golden/tools.json";

/// The tool list in both eras is the committed golden file, byte for byte.
/// `HORSEGIRL_BLESS=1` writes the list that the server sends.
#[test]
fn the_tool_list_is_the_committed_contract_in_both_eras() {
    let list = |era| {
        let answers = session_in(era, &[json!({ "method": "tools/list" })]);
        answers[0]["result"]["tools"].clone()
    };
    let legacy = list(Era::Legacy);
    assert_eq!(list(Era::Modern), legacy);
    let got = serde_json::to_string_pretty(&legacy).unwrap() + "\n";
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(TOOLS);
    if std::env::var_os("HORSEGIRL_BLESS").is_some() {
        std::fs::write(&path, &got).unwrap();
    }
    let want = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        got == want,
        "the tool schemas moved; read the diff, then run \
         HORSEGIRL_BLESS=1 cargo test -p horsegirl --test mcp. They are now:\n{got}"
    );
}

/// The formats that JSON Schema 2020-12 defines. schemars writes others, such
/// as `uint32`, and a client validator warns on each one.
const FORMATS: [&str; 19] = [
    "date-time",
    "date",
    "time",
    "duration",
    "email",
    "idn-email",
    "hostname",
    "idn-hostname",
    "ipv4",
    "ipv6",
    "uri",
    "uri-reference",
    "iri",
    "iri-reference",
    "uuid",
    "uri-template",
    "json-pointer",
    "relative-json-pointer",
    "regex",
];

/// Every `format` keyword under `value`, with the path to it.
fn formats(value: &Value, path: &str, found: &mut Vec<(String, String)>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                // A property that is named `format` holds a schema, not a string.
                if let ("format", Value::String(format)) = (key.as_str(), child) {
                    found.push((path.to_owned(), format.clone()));
                }
                formats(child, &format!("{path}/{key}"), found);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                formats(child, &format!("{path}/{index}"), found);
            }
        }
        _ => {}
    }
}

#[test]
fn no_schema_has_a_format_that_json_schema_does_not_define() {
    let answers = session(&[json!({ "method": "tools/list" })]);
    let mut found = Vec::new();
    formats(&answers[0]["result"]["tools"], "", &mut found);
    let unknown: Vec<_> = found
        .iter()
        .filter(|(_, format)| !FORMATS.contains(&format.as_str()))
        .collect();
    assert!(unknown.is_empty(), "{unknown:?}");
}

#[test]
fn the_format_walk_finds_a_format_that_json_schema_does_not_define() {
    let schema = json!({
        "properties": {
            "format": { "type": "string" },
            "line": { "type": "integer", "format": "uint32" },
            "at": { "type": "string", "format": "date-time" }
        }
    });
    let mut found = Vec::new();
    formats(&schema, "", &mut found);
    let unknown: Vec<_> = found
        .iter()
        .filter(|(_, format)| !FORMATS.contains(&format.as_str()))
        .collect();
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(
        unknown,
        [&("/properties/line".to_owned(), "uint32".to_owned())]
    );
}

/// The `check` result for `Café --> Payments`, as `docs/mcp.md` documents it.
/// The note sits past a two-byte letter, so a column that counts bytes shows.
fn documented_check() -> Value {
    let span = |from: u32, to: u32| json!({ "end": { "column": to, "line": 1 }, "start": { "column": from, "line": 1 } });
    json!({
        "diagnostics": [{
            "code": "N0101",
            "fix": {
                "confidence": "safe",
                "description": "Write the line in the canonical form.",
                "edits": [{ "span": span(6, 9), "text": "->" }]
            },
            "message": "`Café --> Payments` is written the way Mermaid writes it. \
                        horsegirl reads it, and the canonical form is `Café -> Payments`.",
            "severity": "note",
            "span": span(1, 18)
        }],
        "errors": 0,
        "notes": 1,
        "warnings": 0
    })
}

#[test]
fn check_after_a_legacy_initialize_returns_the_documented_result() {
    let calls = [call("check", &json!({ "source": "Café --> Payments\n" }))];
    let answers = session_in(Era::Legacy, &calls);
    assert_eq!(structured(&answers[0]), documented_check());
    // A legacy client gets the legacy shape, with no `resultType`.
    assert!(
        answers[0]["result"].get("resultType").is_none(),
        "{}",
        answers[0]
    );
}

#[test]
fn check_with_modern_meta_after_discover_returns_the_same_result() {
    let calls = [call("check", &json!({ "source": "Café --> Payments\n" }))];
    let answers = session_in(Era::Modern, &calls);
    assert_eq!(structured(&answers[0]), documented_check());
    assert_eq!(
        answers[0]["result"]["resultType"], "complete",
        "{}",
        answers[0]
    );
}

#[test]
fn a_modern_request_without_its_meta_is_refused() {
    // Without this, a modern session that fell back to the legacy path would
    // pass the test above unnoticed.
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 0, "method": "server/discover",
            "params": { "_meta": modern_meta() }
        }),
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": "check", "arguments": { "source": "A -> B\n" } }
        }),
    ];
    let lines: String = input.iter().map(|m| m.to_string() + "\n").collect();
    let out = horsegirl(&["mcp"], &lines);
    let answers: Vec<Value> = text(&out.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let answer = answers
        .iter()
        .find(|a| a["id"] == 1)
        .expect("an answer to 1");
    assert_eq!(answer["error"]["code"], -32602, "{answer}");
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
