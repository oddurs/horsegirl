//! The report of a run, in Markdown: the totals, the diagnostics that the
//! replies hit most often, and the score of each task.

use std::collections::BTreeMap;

use horsegirl_core::codes::CATALOGUE;

use crate::{Answer, Score};

/// How often replies hit one code.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Hits {
    /// Every diagnostic with the code on the first try.
    times: usize,
    /// The replies with at least one on the first try.
    replies: usize,
    /// The replies with at least one left after one `fix`.
    left: usize,
}

/// Each code that a reply hit, the most often first, then by code.
fn hits(scores: &[Score]) -> Vec<(&'static str, Hits)> {
    // A BTreeMap, so equal counts sort by code on every machine.
    let mut by_code: BTreeMap<&'static str, Hits> = BTreeMap::new();
    for answer in scores.iter().filter_map(|score| score.answer.as_ref()) {
        for code in &answer.first {
            by_code.entry(code).or_default().times += 1;
        }
        let mut first = answer.first.clone();
        first.sort_unstable();
        first.dedup();
        for code in first {
            by_code.entry(code).or_default().replies += 1;
        }
        let mut left = answer.left.clone();
        left.sort_unstable();
        left.dedup();
        for code in left {
            by_code.entry(code).or_default().left += 1;
        }
    }
    let mut all: Vec<(&'static str, Hits)> = by_code.into_iter().collect();
    all.sort_by(|(a, x), (b, y)| {
        y.replies
            .cmp(&x.replies)
            .then(y.times.cmp(&x.times))
            .then(a.cmp(b))
    });
    all
}

/// Text for a cell of a Markdown table.
fn cell(text: &str) -> String {
    text.replace('|', "\\|")
}

fn codes(codes: &[&str]) -> String {
    let mut seen: Vec<&str> = Vec::new();
    for code in codes {
        if !seen.contains(code) {
            seen.push(code);
        }
    }
    seen.join(" ")
}

fn valid(valid: bool) -> &'static str {
    if valid { "valid" } else { "invalid" }
}

/// The report of a run named `run`.
#[must_use]
pub fn render(run: &str, scores: &[Score]) -> String {
    let answers: Vec<&Answer> = scores.iter().filter_map(|s| s.answer.as_ref()).collect();
    let total = scores.len();
    let count = |test: fn(&Answer) -> bool| answers.iter().filter(|a| test(a)).count();
    let mut lines = vec![
        format!("# Benchmark report: {run}"),
        String::new(),
        format!(
            "{total} tasks. {} have a reply. A task with no reply scores as a failure.",
            answers.len()
        ),
        String::new(),
        "| Score | Tasks |".to_owned(),
        "| --- | ---: |".to_owned(),
        format!(
            "| Valid on the first try | {} of {total} |",
            count(|a| a.valid_first)
        ),
        format!(
            "| Valid after one `fix` | {} of {total} |",
            count(|a| a.valid_fixed)
        ),
        format!(
            "| Valid after one `fix`, with every expected fact | {} of {total} |",
            count(Answer::correct)
        ),
        String::new(),
        "## The diagnostics that replies hit most often".to_owned(),
        String::new(),
    ];
    let hits = hits(scores);
    if hits.is_empty() {
        lines.push("No reply hit a diagnostic.".to_owned());
    } else {
        lines.extend([
            "Counted on the first try. \"Left\" counts the replies that still have the code after one `fix`.".to_owned(),
            String::new(),
            "| Code | Replies | Times | Left | Meaning |".to_owned(),
            "| --- | ---: | ---: | ---: | --- |".to_owned(),
        ]);
        for (code, hit) in hits {
            let meaning = CATALOGUE
                .iter()
                .find(|entry| entry.code == code)
                .map_or("", |entry| entry.summary);
            lines.push(format!(
                "| `{code}` | {} | {} | {} | {} |",
                hit.replies,
                hit.times,
                hit.left,
                cell(meaning)
            ));
        }
    }
    lines.extend([
        String::new(),
        "## Each task".to_owned(),
        String::new(),
        "\"Facts\" counts the facts of the checker that the file states after one `fix`.".to_owned(),
        String::new(),
        "| Task | Title | First try | After `fix` | Facts | Diagnostics on the first try | First missing fact |".to_owned(),
        "| --- | --- | --- | --- | ---: | --- | --- |".to_owned(),
    ]);
    for score in scores {
        let title = cell(&score.title);
        let Some(answer) = &score.answer else {
            lines.push(format!(
                "| {} | {title} | no reply | no reply | | | |",
                score.id
            ));
            continue;
        };
        let first_missing = answer
            .missing
            .first()
            .map_or_else(String::new, |fact| format!("`{}`", cell(fact)));
        lines.push(format!(
            "| {} | {title} | {} | {} | {}/{} | {} | {first_missing} |",
            score.id,
            valid(answer.valid_first),
            valid(answer.valid_fixed),
            answer.expected - answer.missing.len(),
            answer.expected,
            codes(&answer.first),
        ));
    }
    lines.join("\n") + "\n"
}
