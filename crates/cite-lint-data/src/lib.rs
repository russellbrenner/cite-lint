//! # cite-lint-data
//!
//! Versioned AGLC reference data and the declarative Rule IR (invariant 2:
//! reference data before rules; architecture §5).
//!
//! What: loads an edition's controlled vocabularies (reporters, courts,
//! jurisdictions) and rule entries into validated, lookup-ready form.
//! How: call [`load`] with an edition id (default `"aglc4"`); share the
//! returned [`EditionTables`] via `Arc` across threads (it is immutable).
//! Depends on: `std` only — data files are embedded at compile time, so
//! loading is filesystem- and network-free.
//!
//! This crate sits at the bottom of the dependency graph: nothing in it may
//! depend on `cite-lint-core`, hosts, or surfaces (invariant 1).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod edition;
mod error;
mod ir;
mod minitoml;
mod tables;

pub use edition::{editions, load, EditionMeta, EditionTables};
pub use error::DataError;
pub use ir::{AglcRef, Confidence, RuleIr, Severity, Trigger};
pub use tables::{Court, Jurisdiction, Provenance, Reporter, YearBracket};
