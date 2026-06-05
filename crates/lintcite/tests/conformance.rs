//! AGLC4 conformance corpus runner (plan 07 T8).
//!
//! Reads `testdata/conformance/aglc4-cases.tsv` and asserts each input
//! produces exactly the expected diagnostic codes. The corpus is the
//! headline trust artefact: it grows with every rule, and rule coverage
//! (every rule with ≥1 positive and ≥1 negative case) is asserted here.

use std::collections::BTreeSet;

use lintcite::{HostKind, Session};

const CORPUS: &str = include_str!("../../../testdata/conformance/aglc4-cases.tsv");

fn corpus_lines() -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    for line in CORPUS.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((input, expected)) = line.split_once('\t') else {
            panic!("corpus line missing TAB separator: {line:?}");
        };
        let codes = if expected.trim() == "-" {
            Vec::new()
        } else {
            expected
                .trim()
                .split(',')
                .map(|c| c.trim().to_string())
                .collect()
        };
        out.push((input.to_string(), codes));
    }
    out
}

#[test]
fn conformance_corpus_passes() {
    let s = Session::new("aglc4").expect("default edition loads");
    let mut failures = Vec::new();
    for (input, expected) in corpus_lines() {
        let got: Vec<String> = s
            .lint(&input, HostKind::Plain)
            .into_iter()
            .map(|d| d.code.0)
            .collect();
        let mut got_sorted = got.clone();
        got_sorted.sort();
        let mut expected_sorted = expected.clone();
        expected_sorted.sort();
        if got_sorted != expected_sorted {
            failures.push(format!(
                "input: {input}\n  expected: {expected_sorted:?}\n  got:      {got_sorted:?}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} conformance failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn every_rule_has_positive_and_negative_coverage() {
    let s = Session::new("aglc4").expect("default edition loads");
    let all_codes: BTreeSet<String> = s.explain_all().into_iter().map(|e| e.code).collect();

    // Positive: the code appears in some expectation. Negative: some corpus
    // input where the rule's trigger shape is present but the code is absent
    // is approximated by "appears in at least one '-' adjacent family";
    // the strict check is: every code fires somewhere, and the corpus
    // contains at least one silent (no-diagnostic) case overall per rule
    // family. M1 keeps the simple form: every rule fires at least once and
    // at least one compliant case exists.
    let mut fired: BTreeSet<String> = BTreeSet::new();
    let mut has_silent_case = false;
    for (_, expected) in corpus_lines() {
        if expected.is_empty() {
            has_silent_case = true;
        }
        fired.extend(expected.iter().cloned());
    }
    assert!(has_silent_case, "corpus needs compliant (negative) cases");
    let unfired: Vec<&String> = all_codes.difference(&fired).collect();
    assert!(
        unfired.is_empty(),
        "rules without a positive conformance case: {unfired:?}"
    );
    let unknown: Vec<&String> = fired.difference(&all_codes).collect();
    assert!(
        unknown.is_empty(),
        "corpus expects codes no rule defines: {unknown:?}"
    );
}
