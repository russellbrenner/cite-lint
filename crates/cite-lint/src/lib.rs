//! # cite-lint — the SDK facade
//!
//! The **one behaviour surface** (principle P1): every CLI subcommand, LSP
//! request, MCP tool, and language binding wraps the capabilities exposed
//! here. No surface contains linting logic this crate lacks.
//!
//! What: the capability set — `lint`, `parse`, `explain`, `fix`, `tokens`,
//! `editions` — over a loaded edition.
//! How: build a [`Session`] (or use the zero-config [`lint`] function) and
//! call capabilities; results use the single [`Diagnostic`] model
//! (invariant 5). Sessions are immutable and cheap to share.
//! Depends on: `cite-lint-core`, `cite-lint-host`, `cite-lint-data` — wired
//! together here and only here (architecture §2).
//!
//! Zero-config quickstart (plan 05 T3 — this is a compiled doctest):
//!
//! ```
//! let diagnostics = cite_lint::lint(
//!     "[^1]: Mabo v Queensland (No 2) [1992] 175 CLR 1.",
//! ).expect("default edition loads");
//! assert_eq!(diagnostics.len(), 1);
//! assert_eq!(diagnostics[0].code.0, "AGLC4-CASE-001");
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt;
use std::sync::Arc;

pub use cite_lint_core::{
    AglcRef, Citation, CitationKind, Confidence, Diagnostic, DiagnosticCode, FixIt, RuleRef,
    Severity, SourceRange,
};
pub use cite_lint_host::{host_for_path, HostKind};

use cite_lint_core::{parse, EngineError, RuleSet};
use cite_lint_data::{editions as data_editions, load, DataError, EditionTables};
use cite_lint_host::adapter_for;

/// The closed, versioned capability set (architecture §3). The parity
/// harness and every surface enumerate THIS list — adding a capability
/// without wiring its surfaces fails the parity test (plan 07 T7).
pub const CAPABILITIES: &[&str] = &["lint", "parse", "explain", "fix", "tokens", "editions"];

/// SDK errors. Typed, never panicking (plan 00 T4).
#[derive(Debug)]
pub enum Error {
    /// Edition data failed to load or validate.
    Data(DataError),
    /// The rule set failed to compile (dangling check/fix names).
    Engine(EngineError),
    /// The capability exists in the set but is not yet implemented
    /// (e.g. `tokens` before the LSP slice lands).
    Unimplemented {
        /// The capability name from [`CAPABILITIES`].
        capability: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Data(e) => write!(f, "edition data error: {e}"),
            Error::Engine(e) => write!(f, "rule engine error: {e}"),
            Error::Unimplemented { capability } => {
                write!(f, "capability '{capability}' is not yet implemented")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<DataError> for Error {
    fn from(e: DataError) -> Self {
        Error::Data(e)
    }
}

impl From<EngineError> for Error {
    fn from(e: EngineError) -> Self {
        Error::Engine(e)
    }
}

/// Metadata for one available edition (the `editions` capability).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditionInfo {
    /// Edition id (pass to [`Session::new`]).
    pub id: String,
    /// Human label.
    pub label: String,
    /// The guide's own preferred citation, for attribution.
    pub citation: String,
}

/// The explanation behind a diagnostic code (the `explain` capability).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    /// The stable diagnostic code.
    pub code: String,
    /// The rule's message template.
    pub summary: String,
    /// Severity the rule fires at.
    pub severity: Severity,
    /// Confidence the rule fires with.
    pub confidence: Confidence,
    /// The AGLC4 rule reference.
    pub aglc_ref: AglcRef,
    /// The safe fix transform, when the rule ships one.
    pub fix: Option<String>,
    /// Where the rule derives from (provenance anchor, P9).
    pub provenance: String,
}

/// Result of the `fix` capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixResult {
    /// The document text with all safe, non-overlapping fixes applied.
    pub fixed: String,
    /// How many fixes were applied.
    pub applied: usize,
}

/// An immutable linting session over one edition. Cheap to clone.
#[derive(Clone)]
pub struct Session {
    tables: Arc<EditionTables>,
    rules: Arc<RuleSet>,
}

impl Session {
    /// Load an edition and compile its rule set. The default edition is
    /// `"aglc4"`.
    pub fn new(edition_id: &str) -> Result<Session, Error> {
        let tables = load(edition_id)?;
        let rules = RuleSet::compile(&tables)?;
        Ok(Session {
            tables: Arc::new(tables),
            rules: Arc::new(rules),
        })
    }

    /// Capability `lint`: extract citation spans from `text` via the host
    /// adapter and lint each in isolation (invariant 3). Diagnostics are
    /// returned in deterministic order (by range, then code) with ranges in
    /// host-document bytes.
    pub fn lint(&self, text: &str, host: HostKind) -> Vec<Diagnostic> {
        let adapter = adapter_for(host);
        let mut out = Vec::new();
        for span in adapter.extract(text) {
            let citation = parse(&span.text, &self.tables);
            out.extend(
                self.rules
                    .lint_citation(&citation, &self.tables, span.range.start),
            );
        }
        out.sort_by(|a, b| (a.range.start, &a.code.0).cmp(&(b.range.start, &b.code.0)));
        out
    }

    /// Capability `parse`: one citation string → its typed (possibly
    /// partial) AST. Never errors; malformed input classifies as
    /// `Unclassified` with a reason.
    pub fn parse_citation(&self, citation: &str) -> Citation {
        parse(citation, &self.tables)
    }

    /// Capability `explain`: the rule behind a diagnostic code.
    pub fn explain(&self, code: &str) -> Option<Explanation> {
        self.rules.rule(code).map(|r| Explanation {
            code: r.id.clone(),
            summary: r.message.clone(),
            severity: r.severity,
            confidence: r.confidence,
            aglc_ref: r.aglc_ref.clone(),
            fix: r.fix.clone(),
            provenance: r.provenance.anchor.clone(),
        })
    }

    /// All rules in the loaded edition, for the generated rule catalogue.
    pub fn explain_all(&self) -> Vec<Explanation> {
        self.rules
            .rules()
            .map(|r| Explanation {
                code: r.id.clone(),
                summary: r.message.clone(),
                severity: r.severity,
                confidence: r.confidence,
                aglc_ref: r.aglc_ref.clone(),
                fix: r.fix.clone(),
                provenance: r.provenance.anchor.clone(),
            })
            .collect()
    }

    /// Capability `fix`: apply every safe, non-overlapping fix-it to
    /// `text`. Fixes never change the cited authority (correctness
    /// guardrail); applying them is idempotent (property-tested).
    pub fn fix(&self, text: &str, host: HostKind) -> FixResult {
        let mut fixes: Vec<FixIt> = self
            .lint(text, host)
            .into_iter()
            .filter_map(|d| d.fix)
            .collect();
        // Apply right-to-left so earlier ranges stay valid; skip overlaps.
        fixes.sort_by_key(|f| std::cmp::Reverse(f.range.start));
        let mut fixed = text.to_string();
        let mut applied = 0usize;
        let mut last_start = usize::MAX;
        for f in fixes {
            if f.range.end > last_start || f.range.end > fixed.len() {
                continue; // overlapping or out-of-bounds: skip, never corrupt
            }
            fixed.replace_range(f.range.start..f.range.end, &f.replacement);
            last_start = f.range.start;
            applied += 1;
        }
        FixResult { fixed, applied }
    }

    /// Capability `tokens`: semantic tokens for LSP highlighting. Not yet
    /// implemented (lands with the LSP slice, plan 06 T3) — returns a typed
    /// error, never panics (plan 00 T4).
    pub fn tokens(&self) -> Result<(), Error> {
        Err(Error::Unimplemented {
            capability: "tokens",
        })
    }

    /// Capability `editions`: the editions embedded in this build.
    pub fn editions(&self) -> Result<Vec<EditionInfo>, Error> {
        let mut out = Vec::new();
        for id in data_editions() {
            let t = load(id)?;
            out.push(EditionInfo {
                id: t.meta.id.clone(),
                label: t.meta.label.clone(),
                citation: t.meta.citation.clone(),
            });
        }
        Ok(out)
    }

    /// The loaded edition's id.
    pub fn edition_id(&self) -> &str {
        &self.tables.meta.id
    }
}

/// Zero-config lint: Markdown host, `aglc4` edition (leanness contract:
/// works before any flag is learned).
pub fn lint(text: &str) -> Result<Vec<Diagnostic>, Error> {
    Ok(Session::new("aglc4")?.lint(text, HostKind::Markdown))
}
