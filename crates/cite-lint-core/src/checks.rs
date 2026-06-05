//! Check primitives — the algorithmic vocabulary the Rule IR composes
//! (plan 03 T4/T5).
//!
//! What: each primitive inspects one parsed citation in isolation
//! (invariant 3) and reports whether its rule fires, with template
//! variables and an optional safe fix payload.
//! How: primitives are pure functions consulting the vocabulary tables
//! through their public API; the engine resolves IR `check` names against
//! [`lookup`] at compile time, so a dangling name is a load error, not a
//! runtime surprise.
//! Depends on: [`crate::ast`], `cite-lint-data`.
//!
//! No primitive ever inlines vocabulary (invariant 2) — the
//! ban-inline-vocab architecture test enforces this.

use cite_lint_data::{EditionTables, YearBracket};

use crate::ast::{Citation, CitationKind};
use crate::diagnostic::SourceRange;

/// The result of a fired check: template variables + ranges + fix payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckOutcome {
    /// Fills `{found}` in the message template.
    pub found: String,
    /// Fills `{expected}` in the message template.
    pub expected: String,
    /// Fills `{reporter}` in the message template.
    pub reporter: String,
    /// Citation-local range the diagnostic should cover.
    pub range: SourceRange,
    /// Optional safe fix: citation-local range + replacement text.
    pub fix: Option<(SourceRange, String)>,
}

/// A check primitive: fires with an outcome, or stays silent.
pub(crate) type CheckFn = fn(&Citation, &EditionTables) -> Option<CheckOutcome>;

/// Resolve an IR check name to its primitive. `None` = dangling reference.
pub(crate) fn lookup(name: &str) -> Option<CheckFn> {
    match name {
        "year-bracket-matches-reporter" => Some(year_bracket_matches_reporter),
        "party-separator-is-v" => Some(party_separator_is_v),
        "no-dots-in-reporter" => Some(no_dots_in_reporter),
        "pinpoint-comma-spacing" => Some(pinpoint_comma_spacing),
        "reporter-known" => Some(reporter_known),
        "medium-neutral-square-bracket" => Some(medium_neutral_square_bracket),
        "volume-required-for-round-series" => Some(volume_required_for_round_series),
        _ => None,
    }
}

/// Safe fix transform names a rule may declare (plan 03 T6).
pub(crate) const FIX_NAMES: &[&str] = &[
    "swap-year-bracket",
    "normalise-party-separator",
    "strip-reporter-dots",
    "normalise-pinpoint",
    "square-year-bracket",
];

fn bracket_label(b: YearBracket) -> &'static str {
    match b {
        YearBracket::Round => "round brackets",
        YearBracket::Square => "square brackets",
    }
}

fn bracketed(value: u32, b: YearBracket) -> String {
    match b {
        YearBracket::Round => format!("({value})"),
        YearBracket::Square => format!("[{value}]"),
    }
}

/// AGLC4-CASE-001: a reported citation's year bracket must match the
/// series' organisation (AGLC4 r 2.2.1).
fn year_bracket_matches_reporter(c: &Citation, tables: &EditionTables) -> Option<CheckOutcome> {
    let CitationKind::Reported(r) = &c.kind else {
        return None;
    };
    let reporter = tables.reporter(&r.series.normalised)?;
    if r.year.bracket == reporter.year_bracket {
        return None;
    }
    Some(CheckOutcome {
        found: bracket_label(r.year.bracket).to_string(),
        expected: bracket_label(reporter.year_bracket).to_string(),
        reporter: reporter.abbrev.clone(),
        range: r.year.range,
        fix: Some((r.year.range, bracketed(r.year.value, reporter.year_bracket))),
    })
}

/// AGLC4-CASE-002: parties are separated by an unpunctuated lowercase `v`
/// (AGLC4 r 2.1.11).
fn party_separator_is_v(c: &Citation, _tables: &EditionTables) -> Option<CheckOutcome> {
    let parties = c.parties()?;
    let sep = parties.separator.as_ref()?;
    if sep.raw == "v" {
        return None;
    }
    Some(CheckOutcome {
        found: sep.raw.clone(),
        expected: "v".to_string(),
        reporter: String::new(),
        range: sep.range,
        fix: Some((sep.range, "v".to_string())),
    })
}

/// AGLC4-CASE-003: report series abbreviations take no full stops
/// (AGLC4 r 1.6.1).
fn no_dots_in_reporter(c: &Citation, _tables: &EditionTables) -> Option<CheckOutcome> {
    let CitationKind::Reported(r) = &c.kind else {
        return None;
    };
    if !r.series.raw.contains('.') {
        return None;
    }
    Some(CheckOutcome {
        found: r.series.raw.clone(),
        expected: r.series.normalised.clone(),
        reporter: r.series.normalised.clone(),
        range: r.series.range,
        fix: Some((r.series.range, r.series.normalised.clone())),
    })
}

/// AGLC4-CASE-004: a pinpoint follows the starting page as `, <pinpoint>`.
fn pinpoint_comma_spacing(c: &Citation, _tables: &EditionTables) -> Option<CheckOutcome> {
    let CitationKind::Reported(r) = &c.kind else {
        return None;
    };
    let pin = r.pinpoint.as_ref()?;
    if pin.well_formed {
        return None;
    }
    let content = pin.raw.trim_start_matches([',', ' ']);
    if content.is_empty() {
        return None; // nothing recoverable to fix; stay silent rather than guess
    }
    Some(CheckOutcome {
        found: pin.raw.clone(),
        expected: format!(", {content}"),
        reporter: r.series.normalised.clone(),
        range: pin.range,
        fix: Some((pin.range, format!(", {content}"))),
    })
}

/// AGLC4-CASE-005 (low confidence): the report series is not in the
/// edition vocabulary — surfaced, never guessed at.
fn reporter_known(c: &Citation, tables: &EditionTables) -> Option<CheckOutcome> {
    let CitationKind::Reported(r) = &c.kind else {
        return None;
    };
    if tables.reporter(&r.series.normalised).is_some() {
        return None;
    }
    Some(CheckOutcome {
        found: r.series.normalised.clone(),
        expected: String::new(),
        reporter: r.series.normalised.clone(),
        range: r.series.range,
        fix: None,
    })
}

/// AGLC4-CASE-006: medium-neutral citations take square brackets
/// (AGLC4 r 2.3.1).
fn medium_neutral_square_bracket(c: &Citation, _tables: &EditionTables) -> Option<CheckOutcome> {
    let CitationKind::MediumNeutral(m) = &c.kind else {
        return None;
    };
    if m.year.bracket == YearBracket::Square {
        return None;
    }
    Some(CheckOutcome {
        found: bracketed(m.year.value, m.year.bracket),
        expected: bracketed(m.year.value, YearBracket::Square),
        reporter: m.court_id.normalised.clone(),
        range: m.year.range,
        fix: Some((m.year.range, bracketed(m.year.value, YearBracket::Square))),
    })
}

/// AGLC4-CASE-007: volumed (round-year) series require a volume number.
/// No fix: inventing a volume would change the cited authority.
fn volume_required_for_round_series(c: &Citation, tables: &EditionTables) -> Option<CheckOutcome> {
    let CitationKind::Reported(r) = &c.kind else {
        return None;
    };
    if r.volume.is_some() {
        return None;
    }
    let reporter = tables.reporter(&r.series.normalised)?;
    if reporter.year_bracket != YearBracket::Round {
        return None;
    }
    Some(CheckOutcome {
        found: r.series.normalised.clone(),
        expected: "a volume number before the series".to_string(),
        reporter: reporter.abbrev.clone(),
        range: r.series.range,
        fix: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use cite_lint_data::load;

    fn tables() -> EditionTables {
        load("aglc4").expect("embedded aglc4 edition loads")
    }

    #[test]
    fn every_ir_check_name_resolves() {
        let t = tables();
        for rule in t.rules() {
            assert!(
                lookup(&rule.check).is_some(),
                "IR rule {} references unknown check '{}'",
                rule.id,
                rule.check
            );
        }
    }

    #[test]
    fn every_ir_fix_name_is_known() {
        let t = tables();
        for rule in t.rules() {
            if let Some(fix) = &rule.fix {
                assert!(
                    FIX_NAMES.contains(&fix.as_str()),
                    "IR rule {} references unknown fix '{}'",
                    rule.id,
                    fix
                );
            }
        }
    }

    #[test]
    fn bracket_check_fires_on_square_clr() {
        let t = tables();
        let c = parse("Mabo v Queensland [1992] 175 CLR 1", &t);
        let hit = year_bracket_matches_reporter(&c, &t).expect("must fire");
        assert_eq!(hit.found, "square brackets");
        assert_eq!(hit.expected, "round brackets");
        let (_, replacement) = hit.fix.expect("fix");
        assert_eq!(replacement, "(1992)");
    }

    #[test]
    fn bracket_check_silent_on_compliant_clr() {
        let t = tables();
        let c = parse("Mabo v Queensland (1992) 175 CLR 1", &t);
        assert!(year_bracket_matches_reporter(&c, &t).is_none());
    }

    #[test]
    fn separator_check_fires_on_v_dot_only() {
        let t = tables();
        let bad = parse("Mabo v. Queensland (1992) 175 CLR 1", &t);
        let hit = party_separator_is_v(&bad, &t).expect("must fire");
        assert_eq!(hit.found, "v.");
        let good = parse("Mabo v Queensland (1992) 175 CLR 1", &t);
        assert!(party_separator_is_v(&good, &t).is_none());
    }

    #[test]
    fn dots_check_fix_strips_dots_only() {
        let t = tables();
        let c = parse("Mabo v Queensland (1992) 175 C.L.R. 1", &t);
        let hit = no_dots_in_reporter(&c, &t).expect("must fire");
        let (_, replacement) = hit.fix.expect("fix");
        assert_eq!(replacement, "CLR");
    }

    #[test]
    fn pinpoint_check_normalises_spacing() {
        let t = tables();
        let c = parse("X v Y (2003) 211 CLR 476 492", &t);
        let hit = pinpoint_comma_spacing(&c, &t).expect("must fire");
        let (_, replacement) = hit.fix.expect("fix");
        assert_eq!(replacement, ", 492");
    }

    #[test]
    fn unknown_reporter_fires_low_confidence_check_without_fix() {
        let t = tables();
        let c = parse("X v Y (1992) 12 ZZQ 1", &t);
        let hit = reporter_known(&c, &t).expect("must fire");
        assert_eq!(hit.found, "ZZQ");
        assert!(hit.fix.is_none());
    }

    #[test]
    fn medium_neutral_round_bracket_fires_with_fix() {
        let t = tables();
        let c = parse("Love v Commonwealth (2020) HCA 3", &t);
        let hit = medium_neutral_square_bracket(&c, &t).expect("must fire");
        let (_, replacement) = hit.fix.expect("fix");
        assert_eq!(replacement, "[2020]");
    }

    #[test]
    fn missing_volume_fires_without_fix() {
        let t = tables();
        let c = parse("X v Y (1992) CLR 1", &t);
        let hit = volume_required_for_round_series(&c, &t).expect("must fire");
        assert!(hit.fix.is_none(), "inventing a volume is never safe");
    }
}
