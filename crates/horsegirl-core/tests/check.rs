//! The checks that read the model: typos, things with no link, broken rules,
//! guesses and unknown lenses.
//!
//! Each case states the file, the codes it gives and, where a check carries a
//! fix, the file after the fix.

use horsegirl_core::check::{LENSES, check, of_model};
use horsegirl_core::diagnostic::{Confidence, Diagnostic};
use horsegirl_core::model::lower;
use horsegirl_core::syntax::parse;

const CHECKOUT: &str = include_str!("../../../examples/checkout.horse");

fn codes(diagnostics: &[Diagnostic]) -> Vec<&'static str> {
    diagnostics.iter().map(|d| d.code).collect()
}

/// The one diagnostic with `code`, with the text that its span covers.
fn only<'a>(
    source: &'a str,
    diagnostics: &'a [Diagnostic],
    code: &str,
) -> (&'a Diagnostic, &'a str) {
    let found: Vec<&Diagnostic> = diagnostics.iter().filter(|d| d.code == code).collect();
    assert_eq!(found.len(), 1, "one {code} expected: {diagnostics:#?}");
    (found[0], &source[found[0].span.clone()])
}

#[test]
fn the_reference_file_gives_no_diagnostic_from_the_checks() {
    let model = lower(&parse(CHECKOUT).tree);
    assert_eq!(of_model(CHECKOUT, &model), []);
    // The parser's one warning says that stories arrive in v0.2 (item 55).
    assert_eq!(codes(&check(CHECKOUT)), ["W0010"]);
}

#[test]
fn a_name_one_edit_from_a_known_name_is_a_typo_with_a_safe_fix() {
    let source = "Web app -> Payments\nPayments -> Bank\nOrders -> Paymnts\n";
    let diagnostics = check(source);
    assert_eq!(codes(&diagnostics), ["E0412"]);
    let (typo, spanned) = only(source, &diagnostics, "E0412");
    assert_eq!(spanned, "Paymnts");
    assert!(typo.message.contains("`Payments`"), "{}", typo.message);
    let fix = typo.fix.as_ref().expect("a typo carries a fix");
    assert_eq!(fix.confidence, Confidence::Safe);
    assert_eq!(fix.edits.len(), 1);
    assert_eq!(fix.edits[0].text, "Payments");
    assert_eq!(
        fix.apply(source),
        "Web app -> Payments\nPayments -> Bank\nOrders -> Payments\n"
    );
    assert_eq!(check(&fix.apply(source)), []);
}

#[test]
fn two_letters_swapped_are_one_edit() {
    let source = "A -> Payments\nPayments -> B\nC -> Paymetns\n";
    let diagnostics = check(source);
    let (typo, spanned) = only(source, &diagnostics, "E0412");
    assert_eq!(spanned, "Paymetns");
    assert_eq!(typo.fix.as_ref().unwrap().edits[0].text, "Payments");
}

#[test]
fn the_fix_writes_the_name_as_the_file_writes_it() {
    // The known name holds a colon, so a bare replacement would not parse.
    let source = "\"API: v2\" -> A\nB -> \"API: v2\"\nC -> \"API: v3\"\n";
    let diagnostics = check(source);
    let (typo, spanned) = only(source, &diagnostics, "E0412");
    assert_eq!(spanned, "\"API: v3\"");
    let fixed = typo.fix.as_ref().unwrap().apply(source);
    assert_eq!(
        fixed,
        "\"API: v2\" -> A\nB -> \"API: v2\"\nC -> \"API: v2\"\n"
    );
    assert_eq!(check(&fixed), []);
}

#[test]
fn two_names_each_written_once_are_not_a_typo() {
    // With no name written more often, the check cannot tell which is right.
    let source = "A -> Payments\nB -> Paymnts\n";
    assert_eq!(codes(&check(source)), [""; 0]);
}

#[test]
fn a_name_one_edit_from_two_known_names_has_no_fix() {
    // `Cart` and `Card` are each written twice, so neither is more likely.
    let source = "A -> Cart\nCart -> B\nC -> Card\nCard -> D\nE -> Carl\n";
    let diagnostics = check(source);
    assert_eq!(codes(&diagnostics), ["E0412"]);
    let (typo, spanned) = only(source, &diagnostics, "E0412");
    assert_eq!(spanned, "Carl");
    assert!(typo.message.contains("`Cart`"), "{}", typo.message);
    assert!(typo.message.contains("`Card`"), "{}", typo.message);
    assert_eq!(typo.fix, None);
}

#[test]
fn a_name_one_edit_from_two_known_names_has_no_fix_when_one_is_written_more() {
    // How often a name is written is weak evidence, so it does not choose.
    let source = "A -> Cart\nCart -> B\nCart -> F\nC -> Card\nCard -> D\nE -> Carl\n";
    let diagnostics = check(source);
    let (typo, _) = only(source, &diagnostics, "E0412");
    assert_eq!(typo.fix, None);
}

#[test]
fn sibling_names_are_not_typos() {
    for source in [
        "Service A -> Gateway\nGateway -> Service A\nService B -> Gateway\n",
        "Deploy EC2 #1 -> Log\nLog -> Deploy EC2 #1\nDeploy EC2 #2 -> Log\n",
        "1 -> 2\n2 -> 1\n3 -> 1\n",
    ] {
        assert_eq!(codes(&check(source)), [""; 0], "{source}");
    }
}

#[test]
fn the_sibling_pairs_of_the_corpus_are_not_typos() {
    // From corpus/flowcharts 0143, 0191 and 0198, written as horsegirl.
    for source in [
        "CI -> Deploy EC2 #1\nDeploy EC2 #1 -> Done\nCI -> Deploy EC2 #2\n",
        "Service A -> Sidecar Proxy\nSidecar Proxy -> Service B\nSidecar Proxy -> Service A\n",
        "Request A -> Event loop\nEvent loop -> Request B\nRequest B -> Request C\nRequest C -> Request D\nRequest D -> Request E\nRequest B -> Event loop\n",
    ] {
        assert_eq!(codes(&check(source)), [""; 0], "{source}");
    }
}

#[test]
fn a_name_two_edits_away_is_not_a_typo() {
    let source = "A -> Payments\nPayments -> B\nC -> Paymts\n";
    assert_eq!(codes(&check(source)), [""; 0]);
}

#[test]
fn a_thing_with_no_link_is_a_warning() {
    let source = "A -> B\nLonely\n";
    let diagnostics = check(source);
    assert_eq!(codes(&diagnostics), ["W0107"]);
    let (orphan, spanned) = only(source, &diagnostics, "W0107");
    assert_eq!(spanned, "Lonely");
    assert_eq!(orphan.fix, None);
}

#[test]
fn a_group_needs_no_link_of_its_own_and_its_parts_do() {
    let source = "Services { Orders, Spare }\nOrders -> Postgres\n";
    let diagnostics = check(source);
    assert_eq!(codes(&diagnostics), ["W0107"]);
    assert_eq!(only(source, &diagnostics, "W0107").1, "Spare");
}

#[test]
fn a_thing_that_a_rule_names_is_not_alone() {
    let source = "Web app -> API\nWeb app -x Postgres: never directly\n";
    assert_eq!(codes(&check(source)), [""; 0]);
}

#[test]
fn a_typo_with_no_link_is_reported_once_as_a_typo() {
    let source = "A -> Payments\nPayments -> B\nPaymnts { owner: Commerce }\n";
    assert_eq!(codes(&check(source)), ["E0412"]);
}

#[test]
fn a_link_that_breaks_a_rule_is_an_error() {
    let source = "Web app -x Postgres: never directly\nWeb app -> Postgres: read\n";
    let diagnostics = check(source);
    assert_eq!(codes(&diagnostics), ["E0500"]);
    let (broken, spanned) = only(source, &diagnostics, "E0500");
    assert_eq!(spanned, "Web app -> Postgres: read");
    assert!(broken.message.contains("line 1"), "{}", broken.message);
    assert!(
        broken.message.contains("never directly"),
        "{}",
        broken.message
    );
    assert_eq!(broken.fix, None);
}

#[test]
fn a_fan_out_breaks_a_rule_once_for_each_pair_it_states() {
    let source = "Web app, Mobile app -x Postgres\nWeb app, Mobile app -> Postgres\n";
    assert_eq!(codes(&check(source)), ["E0500", "E0500"]);
}

#[test]
fn a_rule_in_the_other_direction_is_not_broken() {
    let source = "A -x B\nB -> A\n";
    assert_eq!(codes(&check(source)), [""; 0]);
}

#[test]
fn a_guess_is_a_note() {
    let source = "A -> B: maybe ?\nC -x D ?\n";
    let diagnostics = check(source);
    assert_eq!(codes(&diagnostics), ["N0200", "N0200"]);
    assert_eq!(&source[diagnostics[0].span.clone()], "A -> B: maybe ?");
    assert_eq!(&source[diagnostics[1].span.clone()], "C -x D ?");
    assert!(diagnostics.iter().all(|d| d.fix.is_none()));
}

#[test]
fn a_firm_statement_of_a_link_is_not_a_guess() {
    let source = "A -> B ?\nA -> B\n";
    assert_eq!(codes(&check(source)), [""; 0]);
}

#[test]
fn an_unknown_lens_is_an_error_that_lists_the_lenses() {
    let source = "A -> B\nshow bubbles\n";
    let diagnostics = check(source);
    assert_eq!(codes(&diagnostics), ["E0300"]);
    let (unknown, spanned) = only(source, &diagnostics, "E0300");
    assert_eq!(spanned, "bubbles");
    for lens in LENSES {
        assert!(unknown.message.contains(lens), "{}", unknown.message);
    }
    assert_eq!(unknown.fix, None);
}

#[test]
fn a_lens_one_edit_from_a_known_lens_gets_a_safe_fix() {
    let source = "A -> B\nshow flwo\n";
    let diagnostics = check(source);
    let (unknown, _) = only(source, &diagnostics, "E0300");
    let fix = unknown.fix.as_ref().expect("a near lens carries a fix");
    assert_eq!(fix.confidence, Confidence::Safe);
    assert_eq!(fix.apply(source), "A -> B\nshow flow\n");
}

#[test]
fn the_comment_form_is_on_the_language_card_and_reads_as_written() {
    // ADR-130: an agent that reads only the card must know that a line
    // which starts with `#` is a comment, or a name such as `#1 priority`
    // disappears without a diagnostic.
    let stated: Vec<&str> = horsegirl_core::SPEC
        .lines()
        .filter(|line| line.contains('#') && line.contains("comment"))
        .collect();
    assert_eq!(stated.len(), 1, "one card line states the comment form");

    let source = "# a remark -> not a link\n  # an indented remark\nA -> B\n";
    assert_eq!(codes(&check(source)), [""; 0]);
    let model = lower(&parse(source).tree);
    assert!(model.find("A").is_some() && model.find("B").is_some());
    assert!(model.find("a remark").is_none() && model.find("not a link").is_none());
}

#[test]
fn every_lens_is_on_the_language_card() {
    for lens in LENSES {
        assert!(
            horsegirl_core::SPEC.contains(lens),
            "{lens} is not on the card"
        );
    }
    for lens in LENSES {
        let source = format!("A -> B\nshow {lens}\n");
        assert_eq!(codes(&check(&source)), [""; 0], "{lens}");
    }
}

#[test]
fn parse_and_check_diagnostics_come_sorted_by_place() {
    let source = "show bubbles\nA -> B -> C\nLonely\n";
    assert_eq!(codes(&check(source)), ["E0300", "E0001", "W0107"]);
}

#[test]
fn two_runs_give_the_same_diagnostics() {
    let source = "A -> Payments\nPayments -> B\nC -> Paymnts\nLonely\nD -> E ?\nshow flwo\n";
    assert_eq!(check(source), check(source));
}

#[test]
fn a_path_read_as_a_group_gets_a_warning_with_the_quote_fix() {
    let source = "Web app -> GET /users/{id}\nGET /users/{id}\n";
    let diagnostics = check(source);
    let found: Vec<&Diagnostic> = diagnostics.iter().filter(|d| d.code == "W0108").collect();
    assert_eq!(found.len(), 1, "{diagnostics:#?}");
    // Only the line that stands alone is a group to the parser.
    assert_eq!(&source[found[0].span.clone()], "GET /users/{id}");
    assert_eq!(found[0].span.start, source.rfind("GET").unwrap());
    let fix = found[0].fix.as_ref().expect("a path carries a fix");
    assert_eq!(fix.confidence, Confidence::Suggested);
    assert_eq!(fix.edits[0].text, "\"GET /users/{id}\"");
    assert_eq!(
        fix.apply(source),
        "Web app -> GET /users/{id}\n\"GET /users/{id}\"\n"
    );
    assert!(!codes(&check(&fix.apply(source))).contains(&"W0108"));
}

#[test]
fn a_group_with_a_name_that_does_not_end_in_a_slash_gives_no_path_warning() {
    for source in [
        "Services { Orders }\n",
        "Services/ { Orders, Payments }\n",
        "Services/ { owner: Commerce }\n",
        "Services/ { Orders -> Payments }\n",
        "Services/ {\n  Orders\n}\n",
        "\"GET /users/{id}\"\n",
    ] {
        assert!(
            !codes(&check(source)).contains(&"W0108"),
            "{source:?}: {:#?}",
            check(source)
        );
    }
}
