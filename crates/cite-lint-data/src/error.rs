//! Typed errors for the data layer.
//!
//! What: every way loading or validating edition data can fail.
//! How: returned by [`crate::load`] and the internal loaders; never panics.
//! Depends on: nothing beyond `std` (hand-implemented `Display`/`Error` —
//! `thiserror` adoption is deferred with the rest of the dependency set).

use std::fmt;

/// Errors raised while parsing or validating edition data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataError {
    /// The requested edition id is not embedded in this build.
    UnknownEdition {
        /// The edition id that was requested.
        requested: String,
        /// The edition ids that are available.
        available: Vec<String>,
    },
    /// A data file failed to parse as the supported TOML subset.
    Parse {
        /// Which embedded file failed.
        file: &'static str,
        /// 1-based line number of the failure.
        line: usize,
        /// What went wrong.
        message: String,
    },
    /// A table row or rule failed schema validation.
    Validation {
        /// Which embedded file the offending item lives in.
        file: &'static str,
        /// A human-readable identifier for the offending item.
        item: String,
        /// What the schema requires.
        message: String,
    },
    /// A Rule IR entry references vocabulary or primitives that do not exist.
    DanglingReference {
        /// The rule id with the dangling reference.
        rule_id: String,
        /// Description of the missing referent.
        message: String,
    },
    /// Two items share an identifier that must be unique.
    Duplicate {
        /// Which embedded file the duplicate lives in.
        file: &'static str,
        /// The duplicated identifier.
        id: String,
    },
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::UnknownEdition {
                requested,
                available,
            } => write!(
                f,
                "unknown edition '{requested}' (available: {})",
                available.join(", ")
            ),
            DataError::Parse {
                file,
                line,
                message,
            } => {
                write!(f, "{file}:{line}: parse error: {message}")
            }
            DataError::Validation {
                file,
                item,
                message,
            } => {
                write!(f, "{file}: invalid item '{item}': {message}")
            }
            DataError::DanglingReference { rule_id, message } => {
                write!(f, "rule '{rule_id}': dangling reference: {message}")
            }
            DataError::Duplicate { file, id } => {
                write!(f, "{file}: duplicate identifier '{id}'")
            }
        }
    }
}

impl std::error::Error for DataError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_unknown_edition_lists_available() {
        let err = DataError::UnknownEdition {
            requested: "aglc5".to_string(),
            available: vec!["aglc4".to_string()],
        };
        assert_eq!(
            err.to_string(),
            "unknown edition 'aglc5' (available: aglc4)"
        );
    }

    #[test]
    fn display_parse_error_carries_location() {
        let err = DataError::Parse {
            file: "tables/reporters.toml",
            line: 7,
            message: "expected '=' after key".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "tables/reporters.toml:7: parse error: expected '=' after key"
        );
    }
}
