//! The IR rule engine (plan 03 T4): compiled rule set → diagnostics.
//!
//! What: compiles the edition's declarative rules into an executable
//! [`RuleSet`] (resolving check and fix names), then runs them over parsed
//! citations to produce [`Diagnostic`]s.
//! How: `RuleSet::compile` once per edition (share via `Arc`); call
//! [`RuleSet::lint_citation`] per citation. Per-citation isolation holds:
//! a rule sees exactly one citation (invariant 3).
//! Depends on: [`crate::checks`], [`crate::ast`], [`crate::diagnostic`],
//! `lintcite-data`.

use std::fmt;

use lintcite_data::{EditionTables, RuleIr, Trigger};

use crate::ast::{Citation, CitationKind};
use crate::checks::{lookup, CheckFn, FIX_NAMES};
use crate::diagnostic::{Diagnostic, DiagnosticCode, FixIt, RuleRef};

/// Errors raised while compiling the rule set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    /// An IR rule references a check primitive that does not exist.
    UnknownCheck {
        /// The offending rule id.
        rule_id: String,
        /// The unresolved check name.
        check: String,
    },
    /// An IR rule references a fix transform that does not exist.
    UnknownFix {
        /// The offending rule id.
        rule_id: String,
        /// The unresolved fix name.
        fix: String,
    },
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::UnknownCheck { rule_id, check } => {
                write!(f, "rule '{rule_id}': unknown check primitive '{check}'")
            }
            EngineError::UnknownFix { rule_id, fix } => {
                write!(f, "rule '{rule_id}': unknown fix transform '{fix}'")
            }
        }
    }
}

impl std::error::Error for EngineError {}

struct CompiledRule {
    ir: RuleIr,
    check: CheckFn,
}

/// An executable rule set compiled from an edition's Rule IR.
pub struct RuleSet {
    edition_id: String,
    rules: Vec<CompiledRule>,
}

impl RuleSet {
    /// Compile the edition's IR rules, resolving every check and fix name.
    /// A dangling name is a compile error, never a runtime surprise.
    pub fn compile(tables: &EditionTables) -> Result<RuleSet, EngineError> {
        let mut rules = Vec::with_capacity(tables.rules().len());
        for ir in tables.rules() {
            let check = lookup(&ir.check).ok_or_else(|| EngineError::UnknownCheck {
                rule_id: ir.id.clone(),
                check: ir.check.clone(),
            })?;
            if let Some(fix) = &ir.fix {
                if !FIX_NAMES.contains(&fix.as_str()) {
                    return Err(EngineError::UnknownFix {
                        rule_id: ir.id.clone(),
                        fix: fix.clone(),
                    });
                }
            }
            rules.push(CompiledRule {
                ir: ir.clone(),
                check,
            });
        }
        Ok(RuleSet {
            edition_id: tables.meta.id.clone(),
            rules,
        })
    }

    /// The IR entry behind a diagnostic code, for `explain`.
    pub fn rule(&self, code: &str) -> Option<&RuleIr> {
        self.rules.iter().map(|r| &r.ir).find(|r| r.id == code)
    }

    /// All compiled rules, in file order (the generated rule catalogue and
    /// `explain --all` read this).
    pub fn rules(&self) -> impl Iterator<Item = &RuleIr> {
        self.rules.iter().map(|r| &r.ir)
    }

    /// Run every applicable rule over one parsed citation.
    ///
    /// `span_offset` is the citation's byte offset in the host document;
    /// diagnostic and fix ranges are returned in host-document space.
    /// Output order is deterministic: by range start, then code.
    pub fn lint_citation(
        &self,
        citation: &Citation,
        tables: &EditionTables,
        span_offset: usize,
    ) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        for rule in &self.rules {
            if !trigger_matches(rule.ir.trigger, &citation.kind) {
                continue;
            }
            let Some(hit) = (rule.check)(citation, tables) else {
                continue;
            };
            let message = render_message(&rule.ir, &hit);
            let fix = match (&rule.ir.fix, hit.fix) {
                (Some(name), Some((range, replacement))) => Some(FixIt {
                    range: range.offset(span_offset),
                    replacement,
                    description: name.clone(),
                }),
                _ => None,
            };
            out.push(Diagnostic {
                code: DiagnosticCode(rule.ir.id.clone()),
                message,
                rule_ref: RuleRef {
                    edition: self.edition_id.clone(),
                    aglc: rule.ir.aglc_ref.clone(),
                },
                severity: rule.ir.severity,
                range: hit.range.offset(span_offset),
                confidence: rule.ir.confidence,
                fix,
            });
        }
        out.sort_by(|a, b| (a.range.start, &a.code.0).cmp(&(b.range.start, &b.code.0)));
        out
    }
}

fn trigger_matches(trigger: Trigger, kind: &CitationKind) -> bool {
    match trigger {
        Trigger::ReportedCitation => matches!(kind, CitationKind::Reported(_)),
        Trigger::MediumNeutralCitation => {
            matches!(kind, CitationKind::MediumNeutral(_))
        }
        Trigger::AnyCase => matches!(
            kind,
            CitationKind::Reported(_) | CitationKind::MediumNeutral(_)
        ),
        Trigger::AnySpan => true,
    }
}

/// Fill `{found}`/`{expected}`/`{reporter}` and append the AGLC4 rule
/// reference, so every message names the rule it enforces (CLAUDE.md
/// correctness guardrail).
fn render_message(ir: &RuleIr, hit: &crate::checks::CheckOutcome) -> String {
    let body = ir
        .message
        .replace("{found}", &hit.found)
        .replace("{expected}", &hit.expected)
        .replace("{reporter}", &hit.reporter);
    format!("{body} (AGLC4 r {})", ir.aglc_ref.rule)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use lintcite_data::{load, Confidence, Severity};

    fn setup() -> (EditionTables, RuleSet) {
        let tables = load("aglc4").expect("embedded aglc4 edition loads");
        let rules = RuleSet::compile(&tables).expect("rule set compiles");
        (tables, rules)
    }

    #[test]
    fn compiles_the_embedded_edition() {
        let (_, rules) = setup();
        assert!(rules.rules().count() >= 6);
    }

    #[test]
    fn compliant_citation_yields_no_diagnostics() {
        let (t, rules) = setup();
        let c = parse("Mabo v Queensland (No 2) (1992) 175 CLR 1", &t);
        assert_eq!(rules.lint_citation(&c, &t, 0), vec![]);
    }

    #[test]
    fn square_bracket_clr_fires_case_001_with_fix() {
        let (t, rules) = setup();
        let c = parse("Mabo v Queensland (No 2) [1992] 175 CLR 1", &t);
        let diags = rules.lint_citation(&c, &t, 0);
        assert_eq!(diags.len(), 1, "{diags:?}");
        let d = &diags[0];
        assert_eq!(d.code.0, "AGLC4-CASE-001");
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.confidence, Confidence::High);
        assert!(d.message.contains("AGLC4 r 2.2.1"), "{}", d.message);
        let fix = d.fix.as_ref().expect("fix-it");
        assert_eq!(fix.replacement, "(1992)");
    }

    #[test]
    fn span_offset_shifts_ranges_into_host_space() {
        let (t, rules) = setup();
        let c = parse("Mabo v Queensland [1992] 175 CLR 1", &t);
        let at_zero = rules.lint_citation(&c, &t, 0);
        let at_100 = rules.lint_citation(&c, &t, 100);
        assert_eq!(at_zero.len(), 1);
        assert_eq!(at_100.len(), 1);
        assert_eq!(at_100[0].range.start, at_zero[0].range.start + 100);
        let (f0, f100) = (
            at_zero[0].fix.as_ref().expect("fix"),
            at_100[0].fix.as_ref().expect("fix"),
        );
        assert_eq!(f100.range.start, f0.range.start + 100);
    }

    #[test]
    fn multiple_violations_all_fire_in_deterministic_order() {
        let (t, rules) = setup();
        // v. separator + dotted reporter + square bracket on a round series.
        let c = parse("Mabo v. Queensland [1992] 175 C.L.R. 1", &t);
        let diags = rules.lint_citation(&c, &t, 0);
        let codes: Vec<&str> = diags.iter().map(|d| d.code.0.as_str()).collect();
        assert_eq!(
            codes,
            vec!["AGLC4-CASE-002", "AGLC4-CASE-001", "AGLC4-CASE-003"],
            "ordered by range start: separator, year, series"
        );
    }

    #[test]
    fn unknown_reporter_fires_low_confidence_005() {
        let (t, rules) = setup();
        let c = parse("X v Y (1992) 12 ZZQ 1", &t);
        let diags = rules.lint_citation(&c, &t, 0);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code.0, "AGLC4-CASE-005");
        assert_eq!(diags[0].confidence, Confidence::Low);
        assert!(diags[0].fix.is_none());
    }

    #[test]
    fn medium_neutral_round_bracket_fires_006() {
        let (t, rules) = setup();
        let c = parse("Love v Commonwealth (2020) HCA 3", &t);
        let diags = rules.lint_citation(&c, &t, 0);
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert_eq!(diags[0].code.0, "AGLC4-CASE-006");
        assert_eq!(
            diags[0].fix.as_ref().map(|f| f.replacement.as_str()),
            Some("[2020]")
        );
    }

    #[test]
    fn missing_volume_fires_007_without_fix() {
        let (t, rules) = setup();
        let c = parse("X v Y (1992) CLR 1", &t);
        let diags = rules.lint_citation(&c, &t, 0);
        assert!(
            diags.iter().any(|d| d.code.0 == "AGLC4-CASE-007"),
            "{diags:?}"
        );
        let d007 = diags
            .iter()
            .find(|d| d.code.0 == "AGLC4-CASE-007")
            .expect("007");
        assert!(d007.fix.is_none());
    }

    #[test]
    fn unclassified_citation_yields_no_case_diagnostics() {
        let (t, rules) = setup();
        let c = parse("complete gibberish with no year", &t);
        assert_eq!(rules.lint_citation(&c, &t, 0), vec![]);
    }

    #[test]
    fn explain_finds_rules_by_code() {
        let (_, rules) = setup();
        let r = rules.rule("AGLC4-CASE-001").expect("known code");
        assert_eq!(r.aglc_ref.rule, "2.2.1");
        assert!(rules.rule("AGLC4-CASE-999").is_none());
    }
}
