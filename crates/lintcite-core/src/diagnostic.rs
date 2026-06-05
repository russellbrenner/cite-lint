//! The one diagnostic model (invariant 5; architecture §4).
//!
//! What: the single `Diagnostic` type every surface renders. Surfaces
//! translate this model; they never invent diagnostics.
//! How: produced by the rule engine; ranges are byte offsets into the host
//! document. Severity/confidence are re-exported from `lintcite-data` so the
//! IR and the diagnostics share one vocabulary.
//! Depends on: `lintcite-data` only.

pub use lintcite_data::{AglcRef, Confidence, Severity};

/// A stable, append-only diagnostic code (e.g. `AGLC4-CASE-001`).
///
/// Codes are the spine of the docs, the conformance corpus, suppression
/// comments, and SARIF `ruleId`s. They are never reused.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiagnosticCode(pub String);

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A half-open byte range `[start, end)` in host-document space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset.
    pub end: usize,
}

impl SourceRange {
    /// Construct a range; callers guarantee `start <= end`.
    pub fn new(start: usize, end: usize) -> Self {
        SourceRange { start, end }
    }

    /// Shift this range by `offset` bytes (citation-local → host-document).
    pub fn offset(self, offset: usize) -> Self {
        SourceRange {
            start: self.start + offset,
            end: self.end + offset,
        }
    }
}

/// A safe, optional quick-fix attached to a diagnostic.
///
/// Fix-its may only make a citation *more* compliant — never change the
/// cited authority (CLAUDE.md correctness guardrail). When unsure, the rule
/// ships no fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixIt {
    /// The host-document byte range to replace.
    pub range: SourceRange,
    /// The replacement text.
    pub replacement: String,
    /// Human description of the transform (e.g. `swap-year-bracket`).
    pub description: String,
}

/// A machine-readable pointer to the rule a diagnostic enforces, including
/// the edition it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleRef {
    /// Edition id (e.g. `aglc4`).
    pub edition: String,
    /// The AGLC rule reference.
    pub aglc: AglcRef,
}

/// The single diagnostic model shared by every surface (invariant 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Stable diagnostic code.
    pub code: DiagnosticCode,
    /// Human message; names the AGLC4 rule it enforces.
    pub message: String,
    /// Machine-readable rule reference.
    pub rule_ref: RuleRef,
    /// Severity.
    pub severity: Severity,
    /// Host-document byte range the diagnostic covers.
    pub range: SourceRange,
    /// `Low` encodes AGLC ambiguity instead of guessing (never
    /// confident-wrong).
    pub confidence: Confidence,
    /// Optional safe fix.
    pub fix: Option<FixIt>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_offset_shifts_both_ends() {
        let r = SourceRange::new(2, 5).offset(10);
        assert_eq!((r.start, r.end), (12, 15));
    }

    #[test]
    fn code_displays_verbatim() {
        assert_eq!(
            DiagnosticCode("AGLC4-CASE-001".to_string()).to_string(),
            "AGLC4-CASE-001"
        );
    }
}
