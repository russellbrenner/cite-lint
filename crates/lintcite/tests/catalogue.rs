//! Docs-from-code staleness gate (P6, plan 09 T5): `docs/rules.md` is
//! generated from the rule registry; CI fails when it drifts.

use lintcite::{Confidence, Session, Severity};

fn severity_label(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Hint => "hint",
    }
}

fn confidence_label(c: Confidence) -> &'static str {
    match c {
        Confidence::High => "high",
        Confidence::Low => "low",
    }
}

/// Render the catalogue exactly as `docs/rules.md` must contain it.
fn generate() -> String {
    let s = Session::new("aglc4").expect("default edition loads");
    let mut out = String::from(
        "# lintcite rule catalogue (aglc4)\n\n> Generated from the rule \
         registry (P6: docs-from-code). Do not edit by hand — the \
         `rule_catalogue_is_current` test regenerates this content and \
         fails CI when it drifts.\n\n",
    );
    for e in s.explain_all() {
        let fix = e.fix.as_deref().unwrap_or("none");
        out.push_str(&format!(
            "## {} — AGLC4 r {}\n\n- severity: {} · confidence: {} · fix-it: {}\n- anchor: {}\n- provenance: {}\n\n{}\n\n",
            e.code,
            e.aglc_ref.rule,
            severity_label(e.severity),
            confidence_label(e.confidence),
            fix,
            e.aglc_ref.anchor,
            e.provenance,
            e.summary
        ));
    }
    out
}

#[test]
fn rule_catalogue_is_current() {
    let expected = generate();
    let committed = include_str!("../../../docs/rules.md");
    assert!(
        committed == expected,
        "docs/rules.md is stale. Regenerate it with the content below \
         (between the BEGIN/END markers):\nBEGIN\n{expected}END\n"
    );
}
