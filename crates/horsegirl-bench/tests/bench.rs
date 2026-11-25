//! The benchmark: its tasks, its checkers, its adapters and its report.
//!
//! The report of the hand-written run in `bench/runs/hand` is a golden file.
//! To accept a change to it, run:
//!
//! ```sh
//! HORSEGIRL_BLESS=1 cargo test -p horsegirl-bench --test bench
//! ```
//!
//! and read the diff before you commit it.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use horsegirl_bench::checker::{Expectation, expectations, missing};
use horsegirl_bench::{
    Command, Model, Recorded, Recording, Task, extract, load_tasks, prompt, report, run, score,
};
use horsegirl_core::check::check;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tasks() -> Vec<Task> {
    load_tasks(&root().join("bench/tasks")).expect("the tasks load")
}

fn task(checker: &str) -> Task {
    Task {
        id: "001".to_owned(),
        title: "A test".to_owned(),
        description: "A test.".to_owned(),
        checker: checker.to_owned(),
    }
}

/// A model that answers each task with its own checker.
struct Reference;

impl Model for Reference {
    fn reply(&mut self, task: &Task, _prompt: &str) -> io::Result<Option<String>> {
        Ok(Some(task.checker.clone()))
    }
}

/// A directory of its own for one test, removed first so a rerun starts clean.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("horsegirl-bench-{}-{name}", std::process::id()));
    match fs::remove_dir_all(&dir) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
            panic!("cannot clear {}: {error}", dir.display())
        }
        _ => {}
    }
    dir
}

#[test]
fn there_are_one_hundred_tasks_with_ids_in_order() {
    let tasks = tasks();
    assert_eq!(tasks.len(), 100);
    for (n, task) in tasks.iter().enumerate() {
        assert_eq!(task.id, format!("{:03}", n + 1), "{}", task.title);
    }
}

#[test]
fn every_checker_is_a_valid_file_with_no_lone_thing() {
    for task in tasks() {
        let codes: Vec<&str> = check(&task.checker).iter().map(|d| d.code).collect();
        let bad: Vec<&&str> = codes
            .iter()
            .filter(|code| code.starts_with('E') || **code == "W0107")
            .collect();
        assert!(bad.is_empty(), "task {} gives {bad:?}", task.id);
    }
}

/// Each word of a checker that a description must give in backticks, so that
/// a model can write it exactly. A fact may give its key and its value in one
/// pair, as `key: value`.
fn unnamed(task: &Task) -> Vec<String> {
    let named = |word: &str| task.description.contains(&format!("`{word}`"));
    let mut lacking = Vec::new();
    let mut need = |word: &str| {
        if !named(word) && !lacking.iter().any(|known| known == word) {
            lacking.push(word.to_owned());
        }
    };
    for expectation in expectations(&task.checker) {
        match expectation {
            Expectation::Thing(name) => need(&name),
            Expectation::Fact { key, value, .. } => {
                if !named(&format!("{key}: {value}")) {
                    need(&key);
                    need(&value);
                }
            }
            Expectation::Link { labels, .. } => labels.iter().for_each(|label| need(label)),
            Expectation::Rule {
                reason: Some(reason),
                ..
            } => need(&reason),
            Expectation::Part { .. } | Expectation::Rule { .. } | Expectation::Show { .. } => {}
        }
    }
    lacking
}

#[test]
fn every_checker_names_its_words_in_its_description() {
    // A model can only meet a name, a label, a fact or a reason that the
    // description gives it exactly.
    let lacking: Vec<String> = tasks()
        .iter()
        .filter_map(|task| {
            let words = unnamed(task);
            (!words.is_empty()).then(|| format!("{}: {}", task.id, words.join(", ")))
        })
        .collect();
    assert!(
        lacking.is_empty(),
        "these descriptions do not give each word in backticks:\n{}",
        lacking.join("\n")
    );
}

#[test]
fn a_description_must_give_each_label_and_fact() {
    let mut task = task("A -> B: save\nB { owner: Core }\nB -x A: no cycles\n");
    task.description = "`A` asks `B`.".to_owned();
    assert_eq!(unnamed(&task), ["owner", "Core", "save", "no cycles"]);
    task.description = "`A` asks `B` to `save`. `B` has `owner: Core`. `no cycles`.".to_owned();
    assert_eq!(unnamed(&task), Vec::<String>::new());
}

#[test]
fn the_checker_of_each_task_scores_full_marks() {
    let tasks = tasks();
    let scores = run(&tasks, &mut Reference).expect("the reference model answers");
    for score in scores {
        let answer = score.answer.expect("a reply");
        assert!(answer.valid_first, "task {}", score.id);
        assert!(answer.correct(), "task {}: {:?}", score.id, answer.missing);
    }
}

#[test]
fn the_hand_run_matches_its_report() {
    let replies = root().join("bench/runs/hand");
    let scores = run(
        &tasks(),
        &mut Recorded {
            dir: replies.clone(),
        },
    )
    .expect("the run scores");
    let report = report::render("hand", &scores);
    let path = replies.join("report.md");
    if std::env::var_os("HORSEGIRL_BLESS").is_some() {
        fs::write(&path, &report).expect("the report is written");
    }
    let expected = fs::read_to_string(&path).expect("the report exists");
    assert_eq!(
        report, expected,
        "the report changed; run HORSEGIRL_BLESS=1 cargo test -p horsegirl-bench --test bench"
    );
}

#[test]
fn the_hand_run_holds_valid_and_invalid_replies() {
    // The golden report proves the scorer only if it scores both kinds.
    let scores = run(
        &tasks(),
        &mut Recorded {
            dir: root().join("bench/runs/hand"),
        },
    )
    .expect("the run scores");
    let answers: Vec<_> = scores.iter().filter_map(|s| s.answer.as_ref()).collect();
    assert!(answers.iter().any(|a| a.valid_first && a.correct()));
    assert!(answers.iter().any(|a| !a.valid_first && a.valid_fixed));
    assert!(answers.iter().any(|a| !a.valid_fixed));
    assert!(answers.iter().any(|a| a.valid_fixed && !a.correct()));
}

#[test]
fn a_task_reads_its_title_description_and_checker() {
    let task = Task::parse(
        "007-search.task",
        "Search\n\nThe `Box` asks.\n---\nBox -> API\n",
    )
    .expect("a task");
    assert_eq!(task.id, "007");
    assert_eq!(task.title, "Search");
    assert_eq!(task.description, "The `Box` asks.");
    assert_eq!(task.checker, "Box -> API\n");
}

#[test]
fn a_task_without_its_parts_is_refused() {
    let cases = [
        (
            "search.task",
            "Search\n\nText\n---\nA -> B\n",
            "three digits",
        ),
        ("001-a.task", "\nText\n---\nA -> B\n", "title"),
        ("001-a.task", "Title\n\nText\nA -> B\n", "`---`"),
        ("001-a.task", "Title\n---\nA -> B\n", "description"),
        ("001-a.task", "Title\n\nText\n---\n\n", "checker"),
    ];
    for (name, text, says) in cases {
        let error = Task::parse(name, text).expect_err(text);
        assert!(error.message.contains(says), "{error}");
    }
}

#[test]
fn the_prompt_holds_the_card_and_the_description() {
    let task = &tasks()[0];
    let prompt = prompt(task);
    assert!(prompt.starts_with(horsegirl_core::SPEC));
    assert!(prompt.ends_with(&format!("{}\n", task.description)));
    assert!(!prompt.contains(task.checker.trim()));
}

#[test]
fn extract_takes_the_first_fenced_block_or_the_whole_reply() {
    assert_eq!(extract("A -> B\n"), "A -> B\n");
    assert_eq!(
        extract("Here:\n```horsegirl\nA -> B\n```\n```\nC -> D\n```\n"),
        "A -> B\n"
    );
    assert_eq!(extract("```\nA -> B\n"), "A -> B\n");
    assert_eq!(extract("  ```\nA -> B\n  ```\n"), "A -> B\n");
}

#[test]
fn a_checker_ignores_case_and_allows_more_facts() {
    let expected = expectations("Web app -> API: Call\nAPI { owner: Core }\nshow flow\n");
    let answer = "web app -> api: call\napi { owner: core }\nAPI -> Cache\nshow Flow\n";
    assert_eq!(missing(&expected, answer), Vec::<String>::new());
}

#[test]
fn a_checker_names_each_fact_the_answer_lacks() {
    let expected =
        expectations("G { A }\nA -> B: go ?\nA -x C\nB { owner: X }\nshow rings from A\n");
    let answer = "A -> B: go\nB { owner: Y }\nshow rings from B\n";
    assert_eq!(
        missing(&expected, answer),
        [
            "thing G",
            "G { A }",
            "B { owner: X }",
            "thing C",
            "A -> B: go ?",
            "A -x C",
            "show rings from A",
        ]
    );
}

#[test]
fn a_label_must_match_when_the_checker_has_one() {
    let expected = expectations("A -> B: save\n");
    assert_eq!(missing(&expected, "A -> B: write\n"), ["A -> B: save"]);
    assert_eq!(missing(&expected, "A -> B\n"), ["A -> B: save"]);
}

#[test]
fn a_reason_must_match_when_the_checker_has_one() {
    let expected = expectations("A -> B\nB -x A: no cycles\n");
    assert_eq!(
        missing(&expected, "A -> B\nB -x A: no cycles\n"),
        Vec::<String>::new()
    );
    assert_eq!(
        missing(&expected, "A -> B\nB -x A\n"),
        ["B -x A: no cycles"]
    );
}

#[test]
fn a_score_applies_one_fix_before_the_checker() {
    let task = task("Gateway -> API\nAPI { note: \"django :8000\" }\n");
    let answer = score(&task, Some("Gateway -> API\nAPI { note: django :8000 }\n"))
        .answer
        .expect("a reply");
    assert_eq!(answer.first, ["E0004"]);
    assert!(!answer.valid_first);
    assert!(answer.valid_fixed);
    assert!(answer.correct());
}

#[test]
fn a_reply_that_states_no_link_is_not_valid() {
    let task = task("A -> B\n");
    for reply in ["", "```horsegirl\n```\n", "I cannot help with that.\n"] {
        let answer = score(&task, Some(reply)).answer.expect("a reply");
        assert!(!answer.valid_first, "{reply:?}");
        assert!(!answer.valid_fixed, "{reply:?}");
        assert!(!answer.correct(), "{reply:?}");
    }
    let answer = score(&task, Some("A -x B\n")).answer.expect("a reply");
    assert!(answer.valid_first, "a rule alone states a fact");
}

#[test]
fn a_command_with_an_empty_reply_scores_as_invalid() {
    let mut model = Command {
        command: "true".to_owned(),
    };
    let scores = run(&tasks()[..1], &mut model).expect("the command runs");
    let answer = scores[0].answer.as_ref().expect("a reply");
    assert!(!answer.valid_first && !answer.valid_fixed);
}

#[test]
fn a_checker_finds_a_fact_on_any_thing_of_the_name() {
    // `api` and `API` are two things. The answer states the fact on the
    // second, and the checker must not stop at the first.
    let expected = expectations("G { API }\nAPI { owner: Core }\nAPI -> Store\nAPI -x Cache\n");
    let answer = "api -> Cache\nG { API }\nAPI { owner: Core }\nAPI -> Store\nAPI -x Cache\n";
    assert_eq!(missing(&expected, answer), Vec::<String>::new());
}

#[test]
fn no_reply_scores_nothing() {
    assert_eq!(score(&task("A -> B\n"), None).answer, None);
}

#[test]
fn a_command_reads_the_prompt_and_its_replies_are_recorded() {
    let dir = scratch("command");
    let tasks: Vec<Task> = tasks().into_iter().take(2).collect();
    let mut model = Command {
        command: "printf '%s\\n' \"$HORSEGIRL_TASK\"; cat".to_owned(),
    };
    let mut recording = Recording {
        model: &mut model,
        dir: dir.clone(),
    };
    run(&tasks, &mut recording).expect("the command runs");
    for task in &tasks {
        let recorded = fs::read_to_string(dir.join(format!("{}.txt", task.id))).expect("a reply");
        assert_eq!(recorded, format!("{}\n{}", task.id, prompt(task)));
    }
    let again = run(&tasks, &mut Recorded { dir: dir.clone() }).expect("the replies score");
    assert_eq!(again.len(), 2);
    fs::remove_dir_all(dir).expect("the scratch directory is removed");
}

#[test]
fn a_command_that_fails_stops_the_run() {
    let mut model = Command {
        command: "exit 3".to_owned(),
    };
    let error = run(&tasks()[..1], &mut model).expect_err("the command fails");
    assert!(error.to_string().contains("task 001"), "{error}");
}

#[test]
fn the_command_line_scores_a_directory() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_horsegirl-bench"))
        .current_dir(root())
        .args(["score", "bench/runs/hand"])
        .output()
        .expect("the binary runs");
    assert!(output.status.success());
    let expected = fs::read_to_string(root().join("bench/runs/hand/report.md")).expect("a report");
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

#[test]
fn the_command_line_refuses_replies_that_are_not_a_directory() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_horsegirl-bench"))
        .current_dir(root())
        .args(["score", "bench/runs/no-such-run"])
        .output()
        .expect("the binary runs");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert!(String::from_utf8_lossy(&output.stderr).contains("is not a directory"));
}
