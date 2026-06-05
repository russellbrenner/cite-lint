//! Typed citation AST (plan 03; grows per source type, minimal for M1 cases).
//!
//! What: the parsed shape of a single citation, with citation-local byte
//! ranges so diagnostics can be mapped back into host documents.
//! How: produced by [`crate::parser`]; consumed by the rule engine. Malformed
//! input yields a partial AST ([`CitationKind::Unclassified`]) rather than an
//! error — error recovery is a parser property, not an afterthought.
//! Depends on: `cite-lint-data` for the year-bracket vocabulary.

use cite_lint_data::YearBracket;

use crate::diagnostic::SourceRange;

/// The parties portion of a case name (`Mabo v Queensland (No 2)`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyNames {
    /// Raw party text exactly as written.
    pub raw: String,
    /// Citation-local range of the raw party text.
    pub range: SourceRange,
    /// The separator between parties, when one was recognised
    /// (`v`, `v.`, `vs`, `vs.`, `V`). `None` for `Re ...` / `Ex parte ...`
    /// styles, which take no adversarial separator.
    pub separator: Option<Separator>,
}

/// A recognised party separator token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Separator {
    /// The separator exactly as written (e.g. `v.`).
    pub raw: String,
    /// Citation-local range of the separator.
    pub range: SourceRange,
}

/// The year element, with its bracket style as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Year {
    /// The year value (e.g. 1992).
    pub value: u32,
    /// Bracket style found around the year.
    pub bracket: YearBracket,
    /// Citation-local range covering the brackets and year.
    pub range: SourceRange,
}

/// The report-series element of a reported citation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Series {
    /// The series text exactly as written (e.g. `C.L.R.` or `Fam LR`).
    pub raw: String,
    /// The normalised abbreviation: full stops stripped, single spaces
    /// (e.g. `CLR`, `Fam LR`). Used for table lookups.
    pub normalised: String,
    /// Citation-local range of the series text.
    pub range: SourceRange,
}

/// A pinpoint reference following the starting page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pinpoint {
    /// The pinpoint text exactly as written, including its separator
    /// (e.g. `, 30` or ` 492`).
    pub raw: String,
    /// Citation-local range of the pinpoint text.
    pub range: SourceRange,
    /// Whether the pinpoint was correctly introduced by `", "`.
    pub well_formed: bool,
}

/// A reported citation: `Parties (Year) [Vol] Series Page[, Pinpoint]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reported {
    /// Parties, when present.
    pub parties: Option<PartyNames>,
    /// The year element.
    pub year: Year,
    /// The volume number, when the series is organised by volume.
    pub volume: Option<u32>,
    /// The report series.
    pub series: Series,
    /// The starting page.
    pub page: u32,
    /// Optional pinpoint following the page.
    pub pinpoint: Option<Pinpoint>,
}

/// A medium-neutral citation: `Parties [Year] CourtId Number`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediumNeutral {
    /// Parties, when present.
    pub parties: Option<PartyNames>,
    /// The year element.
    pub year: Year,
    /// The unique court identifier as written (e.g. `HCA`).
    pub court_id: Series,
    /// The judgment number.
    pub number: u32,
}

/// The classified shape of one citation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationKind {
    /// A reported case citation.
    Reported(Reported),
    /// A medium-neutral case citation.
    MediumNeutral(MediumNeutral),
    /// Citation-like text the parser could not classify. This is the
    /// low-confidence fall-through (correctness guardrail): the engine never
    /// guesses a classification it cannot support.
    Unclassified {
        /// Why classification failed, for the diagnostic message.
        reason: String,
    },
}

/// One parsed citation: the raw text plus its classified shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    /// The citation text exactly as handed to the parser.
    pub raw: String,
    /// The classified shape (partial on malformed input, never a panic).
    pub kind: CitationKind,
}

impl Citation {
    /// The parties element, for any kind that carries one.
    pub fn parties(&self) -> Option<&PartyNames> {
        match &self.kind {
            CitationKind::Reported(r) => r.parties.as_ref(),
            CitationKind::MediumNeutral(m) => m.parties.as_ref(),
            CitationKind::Unclassified { .. } => None,
        }
    }

    /// True when the citation was classified as some case form.
    pub fn is_case(&self) -> bool {
        matches!(
            self.kind,
            CitationKind::Reported(_) | CitationKind::MediumNeutral(_)
        )
    }
}
