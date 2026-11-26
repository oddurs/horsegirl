//! The checks that read the model: typos, things with no link, broken rules,
//! guesses and unknown lenses.
//!
//! Each case states the file, the codes it gives and, where a check carries a
//! fix, the file after the fix.

use horsegirl_core::check::{LENSES, check, fix_all, of_model};
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
    let source = "\"API: users\" -> A\nB -> \"API: users\"\nC -> \"API: userz\"\n";
    let diagnostics = check(source);
    let (typo, spanned) = only(source, &diagnostics, "E0412");
    assert_eq!(spanned, "\"API: userz\"");
    let fixed = typo.fix.as_ref().unwrap().apply(source);
    assert_eq!(
        fixed,
        "\"API: users\" -> A\nB -> \"API: users\"\nC -> \"API: users\"\n"
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
fn version_names_are_not_typos() {
    let source = "New API -> Billing v2\nOld API -> Billing v1\nBilling v2 -> Ledger\n";
    assert_eq!(codes(&check(source)), [""; 0]);
}

#[test]
fn a_plural_beside_its_singular_is_not_a_typo() {
    // The corpus writes `Agents` beside two nodes called `Agent`: a group and
    // its members, which the author meant as two things.
    let source = "Agent -> Tool\nTool -> Agent\nAgents -> Tool\n";
    assert_eq!(codes(&check(source)), [""; 0]);
    let source = "Agents -> Tool\nTool -> Agents\nAgent -> Tool\n";
    assert_eq!(codes(&check(source)), [""; 0]);
}

#[test]
fn a_misspelt_name_is_still_a_typo() {
    let source = "A -> Billing\nBilling -> B\nC -> Biling\n";
    assert_eq!(codes(&check(source)), ["E0412"]);
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
fn a_thing_in_a_linked_group_is_not_alone_at_any_depth() {
    for source in [
        "API gateway -> Services\nServices { Orders, Payments }\n",
        "Edge -> Cloud\nCloud { Services { Orders, Payments } }\n",
        "Services { Orders, Payments }\nWeb app -x Services: never directly\n",
    ] {
        assert_eq!(codes(&check(source)), [""; 0], "{source}");
    }
}

#[test]
fn a_link_that_w0110_reports_does_not_reach_the_parts_of_its_group() {
    let source = "Services -> Orders\nServices { Orders, Spare }\n";
    let diagnostics = check(source);
    assert_eq!(codes(&diagnostics), ["W0110", "W0107"]);
    assert_eq!(only(source, &diagnostics, "W0107").1, "Spare");

    let source = "Services -> Services\nServices { Orders, Spare }\n";
    let diagnostics = check(source);
    assert_eq!(codes(&diagnostics), ["W0110", "W0107", "W0107"]);
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

// BUG-161: `Archive?` read as a firm link to a thing named `Archive?`.
#[test]
fn a_question_mark_against_the_last_name_gets_a_fix_that_makes_a_guess() {
    let source = "Scheduler -> Report job\nReport job -> Archive?\n";
    let diagnostics = check(source);
    assert_eq!(codes(&diagnostics), ["W0111"]);
    let (warning, spanned) = only(source, &diagnostics, "W0111");
    assert_eq!(spanned, "Archive?");
    assert!(
        warning.message.contains("`Archive ?`"),
        "{}",
        warning.message
    );
    assert!(
        warning.message.contains("`\"Archive?\"`"),
        "{}",
        warning.message
    );
    let fix = warning.fix.as_ref().expect("W0111 carries a fix");
    assert_eq!(fix.confidence, Confidence::Safe);
    let fixed = fix_all(source).text;
    assert_eq!(fixed, "Scheduler -> Report job\nReport job -> Archive ?\n");
    assert_eq!(codes(&check(&fixed)), ["N0200"]);
    let model = lower(&parse(&fixed).tree);
    assert!(model.links[1].guess);
    assert_eq!(model.things[model.links[1].to].name, "Archive");
}

#[test]
fn a_glued_question_mark_on_a_rule_a_list_or_a_citation_line_gets_the_fix() {
    let cases = [
        ("A -x B?\n", "A -x B ?\n"),
        ("A -> B, Web app?\n", "A -> B, Web app ?\n"),
        ("A -> B? @ src/a.rs:3\n", "A -> B ? @ src/a.rs:3\n"),
        ("G {\n  A -> B?\n}\n", "G {\n  A -> B ?\n}\n"),
        ("A -> B?;\n", "A -> B ?\n"),
    ];
    for (source, expected) in cases {
        assert!(codes(&check(source)).contains(&"W0111"), "{source:?}");
        assert_eq!(fix_all(source).text, expected, "{source:?}");
    }
}

#[test]
fn a_name_that_ends_in_a_question_mark_in_quotes_stays_a_name() {
    let source = "A -> \"Archive?\"\n";
    assert_eq!(codes(&check(source)), [""; 0]);
    assert_eq!(fix_all(source).text, source);
    let model = lower(&parse(source).tree);
    assert!(!model.links[0].guess);
    assert_eq!(model.things[model.links[0].to].name, "Archive?");
}

// BUG-161, review: the fix made a firm link to a name that the file writes
// with its `?` elsewhere into a guess to a new thing.
#[test]
fn a_name_that_the_file_writes_with_its_question_mark_elsewhere_gives_no_warning() {
    for source in [
        "Start -> Ok?\nOk? -> Yes\nOk? -> No\n",
        "A -> \"B?\"\nC -> B?\n",
        "C -> B?\nA -> \"B?\"\n",
        "Ok?\nStart -> Ok?\n",
        "G { Ok? }\nStart -> Ok?\n",
    ] {
        assert!(
            !codes(&check(source)).contains(&"W0111"),
            "{source:?}: {:#?}",
            check(source)
        );
        assert_eq!(fix_all(source).text, source, "{source:?}");
    }
}

#[test]
fn a_name_glued_to_its_question_mark_on_two_links_still_gets_the_fix() {
    let source = "A -> Archive?\nB -> Archive?\n";
    assert_eq!(codes(&check(source)), ["W0111", "W0111"]);
    assert_eq!(fix_all(source).text, "A -> Archive ?\nB -> Archive ?\n");
}

#[test]
fn a_question_mark_that_does_not_end_a_link_gives_no_warning() {
    for source in [
        "Valid? -> Yes\n",
        "A -> B: ready?\n",
        "A -> B ?\n",
        "A -> B? ?\n",
        "Ready?\nA -> Ready?, B -> C\n",
        "A { note: why? }\nA -> B\n",
        "G { A, B? }\nA -> B\n",
        "A -> ?\n",
    ] {
        assert!(
            !codes(&check(source)).contains(&"W0111"),
            "{source:?}: {:#?}",
            check(source)
        );
    }
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
    assert_eq!(codes(&check(source)), ["E0300", "E0010", "W0107"]);
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

/// The E0420 errors of a file, each with the text that its span covers.
fn misplaced(source: &str) -> Vec<(Diagnostic, &str)> {
    check(source)
        .into_iter()
        .filter(|d| d.code == "E0420")
        .map(|d| {
            let spanned = &source[d.span.clone()];
            (d, spanned)
        })
        .collect()
}

#[test]
fn a_thing_in_a_second_group_is_an_error_that_names_the_first() {
    let source = "G { A }\nH { A }\nA -> B\n";
    let found = misplaced(source);
    assert_eq!(found.len(), 1, "{found:#?}");
    let (error, spanned) = &found[0];
    assert_eq!(spanned, &"A");
    assert_eq!(error.span.start, "G { A }\nH { ".len());
    assert!(error.message.contains("`G`"), "{}", error.message);
    // Removing the only part would turn `H` from a group into a thing.
    assert_eq!(error.fix, None);
}

#[test]
fn a_group_placed_inside_itself_is_an_error() {
    let source = "A { B }\nB { A }\nA -> C\n";
    let found = misplaced(source);
    assert_eq!(found.len(), 1, "{found:#?}");
    let (error, spanned) = &found[0];
    assert_eq!(spanned, &"A");
    assert_eq!(error.span.start, "A { B }\nB { ".len());
    assert!(error.message.contains("hold itself"), "{}", error.message);

    let found = misplaced("A { A }\nA -> B\n");
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].0.message.contains("hold itself"),
        "{}",
        found[0].0.message
    );
}

#[test]
fn a_part_in_the_braces_after_several_names_is_an_error() {
    let source = "A, B { C }\nC -> D\n";
    let found = misplaced(source);
    assert_eq!(found.len(), 1, "{found:#?}");
    let (error, spanned) = &found[0];
    assert_eq!(spanned, &"C");
    assert!(
        error.message.contains("a part belongs to one group"),
        "{}",
        error.message
    );
    assert!(error.message.contains("`A` and `B`"), "{}", error.message);
    assert_eq!(error.fix, None);
}

#[test]
fn the_reference_file_and_sound_groups_give_no_misplaced_part() {
    assert_eq!(misplaced(CHECKOUT), []);
    for source in [
        // The same group again, and a group that holds the thing deeper.
        "G { A }\nG { A, B }\nA -> B\n",
        "G { H { A } }\nG { A -> B }\n",
        // The holder enters the group on a later line.
        "H { A }\nG { A }\nG { H }\nA -> B\n",
        // A thing first named outside braces, then placed.
        "A -> B\nG { A }\n",
        // Facts after several names place nothing.
        "A, B { owner: X }\nA -> B\n",
        // A story places nothing.
        "G { A }\nstory S {\n  A -> B\n}\n",
    ] {
        assert_eq!(misplaced(source), [], "{source:?}");
    }
}

#[test]
fn a_held_name_among_others_gets_a_suggested_fix_that_removes_it() {
    let source = "G { A }\nH { A, B }\nA -> B\n";
    let found = misplaced(source);
    assert_eq!(found.len(), 1, "{found:#?}");
    let fix = found[0].0.fix.as_ref().expect("the name can be removed");
    assert_eq!(fix.confidence, Confidence::Suggested);
    assert_eq!(fix.apply(source), "G { A }\nH { B }\nA -> B\n");
    assert_eq!(misplaced(&fix.apply(source)), []);

    let source = "G { A }\nH { B, A }\nA -> B\n";
    let fix = misplaced(source)[0]
        .0
        .fix
        .clone()
        .expect("the last name too");
    assert_eq!(fix.apply(source), "G { A }\nH { B }\nA -> B\n");
}

#[test]
fn two_held_names_side_by_side_get_fixes_that_do_not_overlap() {
    let source = "G { A, B }\nH { A, B, C }\nA -> B\nB -> C\n";
    let found = misplaced(source);
    assert_eq!(found.len(), 2, "{found:#?}");
    let fixes: Vec<_> = found.iter().filter_map(|(e, _)| e.fix.as_ref()).collect();
    assert_eq!(fixes.len(), 2, "{found:#?}");
    assert!(fixes[0].span().end <= fixes[1].span().start, "{fixes:#?}");
}

/// Only the writer knows whether a second mention in braces means the same
/// thing, so `fix` never removes one. A safe fix here once removed the second
/// `Server` with no word left, and once left a name written once that E0412
/// then rewrote, so two things became one.
#[test]
fn fix_leaves_a_thing_in_two_groups_for_the_writer() {
    for source in [
        "Server -> sync application\nServer -> async application\n\
         WSGI { Server, sync application }\n\
         ASGI { Server, async application }\n",
        "G { Cart }\nH { Cart, B }\nCarts -> B\nCarts -> G\n",
    ] {
        assert_eq!(misplaced(source).len(), 1, "{source:?}");
        let fixed = fix_all(source);
        assert_eq!(fixed.text, source);
        assert_eq!(fixed.applied, 0);
        assert_eq!(misplaced(&fixed.text).len(), 1, "{source:?}");
    }
}

#[test]
fn a_part_after_several_names_names_the_group_that_holds_it() {
    let message = |source: &str| {
        let found = misplaced(source);
        assert_eq!(found.len(), 1, "{source:?}: {found:#?}");
        found[0].0.message.clone()
    };
    // Earlier braces hold `C`, so these braces leave it in `G`.
    let held = message("G { C }\nA, B { C }\nC -> D\n");
    assert!(held.contains("so `C` is in `G`."), "{held}");
    assert!(!held.contains("`A` only"), "{held}");
    // `A` cannot be inside itself, so no group holds it.
    let nowhere = message("A, B { A }\nA -> D\n");
    assert!(nowhere.contains("so `A` is in no group."), "{nowhere}");
    // The first group holds a new part.
    let first = message("A, B { C }\nC -> D\n");
    assert!(first.contains("so `C` is in `A` only."), "{first}");
}

#[test]
fn a_held_name_in_a_link_inside_braces_has_no_fix() {
    let source = "G { A }\nH { A -> B }\n";
    let found = misplaced(source);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].0.fix, None);
}

#[test]
fn a_label_that_holds_a_second_link_gets_a_warning_that_splits_it() {
    let source = "A -> B: pay -> C: ship\n";
    let diagnostics = check(source);
    let found: Vec<&Diagnostic> = diagnostics.iter().filter(|d| d.code == "W0109").collect();
    assert_eq!(found.len(), 1, "{diagnostics:#?}");
    assert_eq!(&source[found[0].span.clone()], "-> C: ship");
    let fix = found[0].fix.as_ref().expect("the warning carries a fix");
    assert_eq!(fix.confidence, Confidence::Suggested);
    let repaired = fix.apply(source);
    assert_eq!(repaired, "A -> B: pay\nB -> C: ship\n");
    assert_eq!(codes(&check(&repaired)), Vec::<&str>::new());
    // The second link keeps the indent of the first.
    let nested = "G {\n  A -> B: pay -> C: ship\n}\n";
    let fix = check(nested)
        .into_iter()
        .find(|d| d.code == "W0109")
        .and_then(|d| d.fix)
        .expect("the nested warning carries a fix");
    assert_eq!(fix.apply(nested), "G {\n  A -> B: pay\n  B -> C: ship\n}\n");
}

#[test]
fn a_split_warning_for_a_marked_link_gives_no_fix() {
    for source in [
        "A -> B: pay -> C: ship ?\n",
        "A -> B: pay -> C: ship @ docs/a.md:3\n",
    ] {
        let diagnostics = check(source);
        let found: Vec<&Diagnostic> = diagnostics.iter().filter(|d| d.code == "W0109").collect();
        assert_eq!(found.len(), 1, "{source:?}: {diagnostics:#?}");
        assert_eq!(found[0].fix, None, "{source:?}");
    }
}

#[test]
fn a_label_with_an_arrow_and_no_colon_after_it_gives_no_warning() {
    for source in [
        "A -> B: calls -> returns\n",
        "A -> B: \"pay -> C: ship\"\n",
        "A -> B: \"x\" -> C\n",
        "A -> B: pay: ship\n",
    ] {
        assert!(
            !codes(&check(source)).contains(&"W0109"),
            "{source:?}: {:#?}",
            check(source)
        );
    }
}

#[test]
fn a_link_between_a_group_and_what_it_holds_is_a_warning() {
    let source = "Services {\n  Orders\n  Inner {\n    Payments\n  }\n}\n\
                  Orders -> Services\nServices -> Payments\nServices -> Services\n\
                  Web -> Services\nOrders -> Payments\nInner -> Orders\n";
    let diagnostics = check(source);
    assert_eq!(codes(&diagnostics), ["W0110", "W0110", "W0110"]);
    let spans: Vec<&str> = diagnostics
        .iter()
        .map(|d| &source[d.span.clone()])
        .collect();
    assert_eq!(
        spans,
        [
            "Orders -> Services",
            "Services -> Payments",
            "Services -> Services"
        ]
    );
    assert_eq!(
        diagnostics[0].message,
        "`Orders -> Services` links the group `Services` to `Orders`, which it holds, so the picture leaves the link out. Link a thing outside `Services` to `Services`, or link two things inside it."
    );
    assert_eq!(diagnostics[0].fix, None);
    assert!(
        diagnostics[2]
            .message
            .contains("the group `Services` to itself")
    );
}
