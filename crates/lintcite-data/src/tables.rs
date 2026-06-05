//! Controlled-vocabulary table schema (invariant 2: reference data before rules).
//!
//! What: typed rows for reporters, courts, and jurisdictions, each carrying
//! mandatory provenance (P9: provenance or it didn't happen).
//! How: loaded from the TOML-subset files under `data/editions/<id>/tables/`
//! by [`crate::load`]; consumed by `lintcite-core` through the compiled
//! [`crate::EditionTables`] lookups.
//! Depends on: [`crate::minitoml`], [`crate::error`].

use crate::error::DataError;
use crate::minitoml::{self, Table};

/// Where a table row came from and who has vouched for it (P9).
///
/// Hand-authored rows record `method = "hand-authored"` and an `anchor`
/// naming the AGLC4 location they derive from (e.g. an appendix or rule
/// number). Ingested rows (plan 02) will record the extraction tool and PDF
/// page. The schema makes all three fields mandatory so provenance can never
/// silently rot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// How the row was produced (e.g. `hand-authored`, `ingest-v1`).
    pub method: String,
    /// Where in the source edition the fact lives (rule/appendix anchor).
    pub anchor: String,
    /// Who reviewed the row (or a pending-review marker).
    pub reviewer: String,
}

impl Provenance {
    fn from_table(file: &'static str, item: &str, t: &Table) -> Result<Self, DataError> {
        let p = minitoml::req_table(file, item, t, "provenance")?;
        Ok(Provenance {
            method: minitoml::req_str(file, item, p, "method")?.to_string(),
            anchor: minitoml::req_str(file, item, p, "anchor")?.to_string(),
            reviewer: minitoml::req_str(file, item, p, "reviewer")?.to_string(),
        })
    }
}

/// Whether a report series encloses the year in round or square brackets.
///
/// AGLC4 rule 2.2.1: series organised by volume number take round brackets
/// around the year; series organised by year (where the year identifies the
/// volume) take square brackets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YearBracket {
    /// `(1992) 175 CLR 1` — independent volume numbers.
    Round,
    /// `[2015] VR 100` style — the year is the volume identifier.
    Square,
}

impl YearBracket {
    fn parse(file: &'static str, item: &str, raw: &str) -> Result<Self, DataError> {
        match raw {
            "round" => Ok(YearBracket::Round),
            "square" => Ok(YearBracket::Square),
            other => Err(DataError::Validation {
                file,
                item: item.to_string(),
                message: format!("year_bracket must be 'round' or 'square', got '{other}'"),
            }),
        }
    }
}

/// A law report series row from `tables/reporters.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reporter {
    /// Canonical abbreviation as cited (e.g. `CLR`).
    pub abbrev: String,
    /// Full series name (e.g. `Commonwealth Law Reports`).
    pub full: String,
    /// Bracket style the series takes around the year (AGLC4 r 2.2.1).
    pub year_bracket: YearBracket,
    /// Court the series is authorised for, when single-court (court id).
    pub court: Option<String>,
    /// Jurisdiction abbreviation (e.g. `Cth`), when jurisdiction-specific.
    pub jurisdiction: Option<String>,
    /// Mandatory provenance.
    pub provenance: Provenance,
}

/// A court row from `tables/courts.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Court {
    /// Unique court identifier as used in medium-neutral citations (`HCA`).
    pub id: String,
    /// Full court name (`High Court of Australia`).
    pub full: String,
    /// Jurisdiction abbreviation (e.g. `Cth`).
    pub jurisdiction: String,
    /// Whether the court issues medium-neutral citations with this id.
    pub medium_neutral: bool,
    /// Mandatory provenance.
    pub provenance: Provenance,
}

/// A jurisdiction row from `tables/jurisdictions.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jurisdiction {
    /// Abbreviation as cited in legislation (`Cth`, `Vic`).
    pub abbrev: String,
    /// Full name (`Commonwealth`).
    pub full: String,
    /// Mandatory provenance.
    pub provenance: Provenance,
}

/// Parse `tables/reporters.toml` content.
pub(crate) fn parse_reporters(file: &'static str, src: &str) -> Result<Vec<Reporter>, DataError> {
    let root = minitoml::parse(file, src)?;
    let mut out = Vec::new();
    for row in minitoml::tables(&root, "reporter") {
        let abbrev = minitoml::req_str(file, "reporter", row, "abbrev")?;
        let item = format!("reporter '{abbrev}'");
        out.push(Reporter {
            abbrev: abbrev.to_string(),
            full: minitoml::req_str(file, &item, row, "full")?.to_string(),
            year_bracket: YearBracket::parse(
                file,
                &item,
                minitoml::req_str(file, &item, row, "year_bracket")?,
            )?,
            court: minitoml::opt_str(row, "court").map(str::to_string),
            jurisdiction: minitoml::opt_str(row, "jurisdiction").map(str::to_string),
            provenance: Provenance::from_table(file, &item, row)?,
        });
    }
    if out.is_empty() {
        return Err(DataError::Validation {
            file,
            item: "reporters".to_string(),
            message: "no [[reporter]] rows found".to_string(),
        });
    }
    Ok(out)
}

/// Parse `tables/courts.toml` content.
pub(crate) fn parse_courts(file: &'static str, src: &str) -> Result<Vec<Court>, DataError> {
    let root = minitoml::parse(file, src)?;
    let mut out = Vec::new();
    for row in minitoml::tables(&root, "court") {
        let id = minitoml::req_str(file, "court", row, "id")?;
        let item = format!("court '{id}'");
        let medium_neutral = match row.get("medium_neutral") {
            Some(crate::minitoml::Item::Bool(b)) => *b,
            None => false,
            _ => {
                return Err(DataError::Validation {
                    file,
                    item,
                    message: "medium_neutral must be a boolean".to_string(),
                })
            }
        };
        out.push(Court {
            id: id.to_string(),
            full: minitoml::req_str(file, &item, row, "full")?.to_string(),
            jurisdiction: minitoml::req_str(file, &item, row, "jurisdiction")?.to_string(),
            medium_neutral,
            provenance: Provenance::from_table(file, &item, row)?,
        });
    }
    if out.is_empty() {
        return Err(DataError::Validation {
            file,
            item: "courts".to_string(),
            message: "no [[court]] rows found".to_string(),
        });
    }
    Ok(out)
}

/// Parse `tables/jurisdictions.toml` content.
pub(crate) fn parse_jurisdictions(
    file: &'static str,
    src: &str,
) -> Result<Vec<Jurisdiction>, DataError> {
    let root = minitoml::parse(file, src)?;
    let mut out = Vec::new();
    for row in minitoml::tables(&root, "jurisdiction") {
        let abbrev = minitoml::req_str(file, "jurisdiction", row, "abbrev")?;
        let item = format!("jurisdiction '{abbrev}'");
        out.push(Jurisdiction {
            abbrev: abbrev.to_string(),
            full: minitoml::req_str(file, &item, row, "full")?.to_string(),
            provenance: Provenance::from_table(file, &item, row)?,
        });
    }
    if out.is_empty() {
        return Err(DataError::Validation {
            file,
            item: "jurisdictions".to_string(),
            message: "no [[jurisdiction]] rows found".to_string(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const F: &str = "tables/test.toml";

    fn provenance_block() -> &'static str {
        "[reporter.provenance]\nmethod = \"hand-authored\"\nanchor = \"Appendix A\"\nreviewer = \"test\"\n"
    }

    #[test]
    fn reporter_row_parses() {
        let src = format!(
            "[[reporter]]\nabbrev = \"CLR\"\nfull = \"Commonwealth Law Reports\"\nyear_bracket = \"round\"\ncourt = \"HCA\"\njurisdiction = \"Cth\"\n{}",
            provenance_block()
        );
        let rows = parse_reporters(F, &src).expect("parse");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].abbrev, "CLR");
        assert_eq!(rows[0].year_bracket, YearBracket::Round);
        assert_eq!(rows[0].court.as_deref(), Some("HCA"));
    }

    #[test]
    fn reporter_missing_provenance_is_rejected() {
        let src = "[[reporter]]\nabbrev = \"CLR\"\nfull = \"Commonwealth Law Reports\"\nyear_bracket = \"round\"\n";
        let err = parse_reporters(F, src).expect_err("must reject");
        assert!(matches!(err, DataError::Validation { .. }), "{err}");
    }

    #[test]
    fn reporter_bad_bracket_is_rejected() {
        let src = format!(
            "[[reporter]]\nabbrev = \"X\"\nfull = \"X Reports\"\nyear_bracket = \"curly\"\n{}",
            provenance_block()
        );
        let err = parse_reporters(F, &src).expect_err("must reject");
        assert!(err.to_string().contains("round"), "{err}");
    }

    #[test]
    fn empty_reporter_file_is_rejected() {
        let err = parse_reporters(F, "# nothing here\n").expect_err("reject");
        assert!(err.to_string().contains("no [[reporter]] rows"), "{err}");
    }

    #[test]
    fn court_defaults_medium_neutral_false() {
        let src = "[[court]]\nid = \"X\"\nfull = \"X Court\"\njurisdiction = \"Cth\"\n[court.provenance]\nmethod = \"hand-authored\"\nanchor = \"r 2.3\"\nreviewer = \"test\"\n";
        let rows = parse_courts(F, src).expect("parse");
        assert!(!rows[0].medium_neutral);
    }
}
