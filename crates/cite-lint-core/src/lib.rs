//! # cite-lint-core
//!
//! The deterministic citation engine (plan 03): citation string + edition
//! tables → typed AST → diagnostics.
//!
//! What: lexes, parses (with error recovery), classifies, and lints a single
//! citation in isolation (invariant 3 — cross-citation logic belongs to the
//! document pass, added at M4).
//! How: compile a [`RuleSet`] once per edition, then call
//! [`RuleSet::lint_citation`] per citation with [`parse`]'s output. Same
//! input + edition ⇒ identical diagnostics, always (P3 — determinism is
//! property-tested).
//! Depends on: `cite-lint-data` ONLY (invariant 1: core never sees a host,
//! a surface, or an edition-by-name).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod ast;
mod checks;
mod classify;
mod diagnostic;
mod engine;
mod lexer;
mod parser;

pub use ast::{
    Citation, CitationKind, MediumNeutral, PartyNames, Pinpoint, Reported, Separator, Series, Year,
};
pub use diagnostic::{
    AglcRef, Confidence, Diagnostic, DiagnosticCode, FixIt, RuleRef, Severity, SourceRange,
};
pub use engine::{EngineError, RuleSet};
pub use parser::parse;
