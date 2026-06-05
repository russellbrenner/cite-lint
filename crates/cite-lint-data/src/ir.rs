//! Declarative Rule IR schema (architecture §5: rules are mostly data too).
//!
//! What: the typed form of `data/editions/<id>/rules/*.ir.toml` — each entry
//! binds a trigger (which AST shapes the rule applies to) to a named check
//! primitive, a severity, a message template, a mandatory AGLC4 reference,
//! and an optional safe fix transform.
//! How: loaded and structurally validated here; `cite-lint-core` compiles the
//! entries into an executable rule set and resolves `check`/`fix` names
//! against its primitive registry (the dependency stays one-way: core → data,
//! never data → core).
//! Depends on: [`crate::minitoml`], [`crate::error`], [`crate::tables`].
//!
//! The IR references vocabulary tables; it never inlines them (invariant 2).
//! The check-primitive vocabulary is deliberately small (plan 01 lean note):
//! an op is added only when a real AGLC rule needs it. R-DATA-1 (the
//! 10-rule expressiveness spike) is partially advanced by the seed rule set
//! and remains open for the full case subset.

use crate::error::DataError;
use crate::minitoml::{self, Table};
use crate::tables::Provenance;

/// Diagnostic severity, shared verbatim with the `Diagnostic` model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// A rule violation.
    Error,
    /// Probably wrong, but with a conceivable compliant reading.
    Warning,
    /// Advisory only.
    Info,
    /// Style hint.
    Hint,
}

impl Severity {
    fn parse(file: &'static str, item: &str, raw: &str) -> Result<Self, DataError> {
        match raw {
            "error" => Ok(Severity::Error),
            "warning" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            "hint" => Ok(Severity::Hint),
            other => Err(DataError::Validation {
                file,
                item: item.to_string(),
                message: format!("unknown severity '{other}'"),
            }),
        }
    }
}

/// How sure the engine is when this rule fires (correctness guardrail:
/// AGLC ambiguity is encoded as `Low`, never as a confident guess).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// The rule's trigger and check are unambiguous.
    High,
    /// The diagnostic encodes uncertainty (e.g. "could not classify").
    Low,
}

impl Confidence {
    fn parse(file: &'static str, item: &str, raw: &str) -> Result<Self, DataError> {
        match raw {
            "high" => Ok(Confidence::High),
            "low" => Ok(Confidence::Low),
            other => Err(DataError::Validation {
                file,
                item: item.to_string(),
                message: format!("unknown confidence '{other}'"),
            }),
        }
    }
}

/// Which parsed-citation shapes a rule applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// Reported citations: `Name v Name (1992) 175 CLR 1`.
    ReportedCitation,
    /// Medium-neutral citations: `Name v Name [2020] HCA 1`.
    MediumNeutralCitation,
    /// Any citation the parser classified as a case.
    AnyCase,
    /// Any citation span, classified or not.
    AnySpan,
}

impl Trigger {
    fn parse(file: &'static str, item: &str, raw: &str) -> Result<Self, DataError> {
        match raw {
            "reported-citation" => Ok(Trigger::ReportedCitation),
            "medium-neutral-citation" => Ok(Trigger::MediumNeutralCitation),
            "any-case" => Ok(Trigger::AnyCase),
            "any-span" => Ok(Trigger::AnySpan),
            other => Err(DataError::Validation {
                file,
                item: item.to_string(),
                message: format!("unknown trigger '{other}'"),
            }),
        }
    }
}

/// A machine-readable pointer to the AGLC4 rule a diagnostic enforces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AglcRef {
    /// Rule number as printed in the guide (e.g. `2.2.4`).
    pub rule: String,
    /// Human-readable anchor (part/section name) for orientation.
    pub anchor: String,
}

/// One declarative rule entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleIr {
    /// Stable, append-only diagnostic code (e.g. `AGLC4-CASE-001`).
    pub id: String,
    /// Which citation shapes the rule runs against.
    pub trigger: Trigger,
    /// Name of the check primitive the engine must run (resolved by core).
    pub check: String,
    /// Severity carried into the diagnostic.
    pub severity: Severity,
    /// Confidence carried into the diagnostic.
    pub confidence: Confidence,
    /// Message template. Placeholders `{found}` / `{expected}` are filled by
    /// the check primitive; the AGLC rule reference is appended by the
    /// renderer so every message names its rule (CLAUDE.md guardrail).
    pub message: String,
    /// Mandatory AGLC4 reference.
    pub aglc_ref: AglcRef,
    /// Optional safe fix transform name (resolved by core; safe-only).
    pub fix: Option<String>,
    /// Mandatory provenance (P9).
    pub provenance: Provenance,
}

/// Validate a diagnostic code: `AGLC4-<AREA>-<NNN>`.
fn validate_code(file: &'static str, id: &str) -> Result<(), DataError> {
    let mut parts = id.split('-');
    let edition = parts.next().unwrap_or("");
    let area = parts.next().unwrap_or("");
    let num = parts.next().unwrap_or("");
    let ok = edition == "AGLC4"
        && !area.is_empty()
        && area.chars().all(|c| c.is_ascii_uppercase())
        && num.len() == 3
        && num.chars().all(|c| c.is_ascii_digit())
        && parts.next().is_none();
    if !ok {
        return Err(DataError::Validation {
            file,
            item: id.to_string(),
            message: "rule id must match AGLC4-<AREA>-<NNN>".to_string(),
        });
    }
    Ok(())
}

/// Parse a `rules/*.ir.toml` file into validated [`RuleIr`] entries.
pub(crate) fn parse_rules(file: &'static str, src: &str) -> Result<Vec<RuleIr>, DataError> {
    let root = minitoml::parse(file, src)?;
    let mut out: Vec<RuleIr> = Vec::new();
    for row in minitoml::tables(&root, "rule") {
        let id = minitoml::req_str(file, "rule", row, "id")?.to_string();
        validate_code(file, &id)?;
        if out.iter().any(|r| r.id == id) {
            return Err(DataError::Duplicate { file, id });
        }
        let item = format!("rule '{id}'");
        let aglc = minitoml::req_table(file, &item, row, "aglc_ref")?;
        out.push(RuleIr {
            trigger: Trigger::parse(file, &item, minitoml::req_str(file, &item, row, "trigger")?)?,
            check: minitoml::req_str(file, &item, row, "check")?.to_string(),
            severity: Severity::parse(
                file,
                &item,
                minitoml::req_str(file, &item, row, "severity")?,
            )?,
            confidence: Confidence::parse(
                file,
                &item,
                minitoml::req_str(file, &item, row, "confidence")?,
            )?,
            message: minitoml::req_str(file, &item, row, "message")?.to_string(),
            aglc_ref: AglcRef {
                rule: minitoml::req_str(file, &item, aglc, "rule")?.to_string(),
                anchor: minitoml::req_str(file, &item, aglc, "anchor")?.to_string(),
            },
            fix: minitoml::opt_str(row, "fix").map(str::to_string),
            provenance: parse_rule_provenance(file, &item, row)?,
            id,
        });
    }
    if out.is_empty() {
        return Err(DataError::Validation {
            file,
            item: "rules".to_string(),
            message: "no [[rule]] rows found".to_string(),
        });
    }
    Ok(out)
}

fn parse_rule_provenance(
    file: &'static str,
    item: &str,
    row: &Table,
) -> Result<Provenance, DataError> {
    let p = minitoml::req_table(file, item, row, "provenance")?;
    Ok(Provenance {
        method: minitoml::req_str(file, item, p, "method")?.to_string(),
        anchor: minitoml::req_str(file, item, p, "anchor")?.to_string(),
        reviewer: minitoml::req_str(file, item, p, "reviewer")?.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const F: &str = "rules/test.ir.toml";

    fn rule_src(id: &str) -> String {
        format!(
            "[[rule]]\nid = \"{id}\"\ntrigger = \"reported-citation\"\ncheck = \"year-bracket-matches-reporter\"\nseverity = \"error\"\nconfidence = \"high\"\nmessage = \"wrong bracket\"\nfix = \"swap-year-bracket\"\n[rule.aglc_ref]\nrule = \"2.2.1\"\nanchor = \"Part 2 - Cases\"\n[rule.provenance]\nmethod = \"hand-authored\"\nanchor = \"r 2.2.1\"\nreviewer = \"test\"\n"
        )
    }

    #[test]
    fn valid_rule_parses() {
        let rules = parse_rules(F, &rule_src("AGLC4-CASE-001")).expect("parse");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "AGLC4-CASE-001");
        assert_eq!(rules[0].trigger, Trigger::ReportedCitation);
        assert_eq!(rules[0].severity, Severity::Error);
        assert_eq!(rules[0].fix.as_deref(), Some("swap-year-bracket"));
        assert_eq!(rules[0].aglc_ref.rule, "2.2.1");
    }

    #[test]
    fn malformed_code_is_rejected() {
        for bad in [
            "CASE-001",
            "AGLC4-case-001",
            "AGLC4-CASE-1",
            "AGLC4-CASE-0001",
        ] {
            assert!(
                parse_rules(F, &rule_src(bad)).is_err(),
                "should reject {bad}"
            );
        }
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let src = format!(
            "{}{}",
            rule_src("AGLC4-CASE-001"),
            rule_src("AGLC4-CASE-001")
        );
        let err = parse_rules(F, &src).expect_err("duplicate");
        assert!(matches!(err, DataError::Duplicate { .. }), "{err}");
    }

    #[test]
    fn missing_aglc_ref_is_rejected() {
        let src = "[[rule]]\nid = \"AGLC4-CASE-001\"\ntrigger = \"any-case\"\ncheck = \"x\"\nseverity = \"error\"\nconfidence = \"high\"\nmessage = \"m\"\n[rule.provenance]\nmethod = \"hand-authored\"\nanchor = \"a\"\nreviewer = \"t\"\n";
        let err = parse_rules(F, src).expect_err("must reject");
        assert!(err.to_string().contains("aglc_ref"), "{err}");
    }
}
