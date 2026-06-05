//! Edition loading and compiled lookups.
//!
//! What: `load("aglc4")` → a validated, lookup-ready [`EditionTables`].
//! How: edition data files are embedded at compile time with `include_str!`,
//! so loading needs no filesystem, no network, and no warmup — the
//! near-zero-cold-start property (architecture §6) holds by construction.
//! Lookups are `BTreeMap`-backed (deterministic iteration); the FST/PHF
//! upgrade is gated on R-ARCH-2 and changes nothing observable.
//! Depends on: [`crate::tables`], [`crate::ir`], [`crate::minitoml`].

use std::collections::BTreeMap;

use crate::error::DataError;
use crate::ir::{self, RuleIr};
use crate::minitoml;
use crate::tables::{self, Court, Jurisdiction, Reporter};

/// Embedded data files for the AGLC4 edition (repo `data/` directory).
mod aglc4_files {
    pub const META: (&str, &str) = (
        "data/editions/aglc4/meta.toml",
        include_str!("../../../data/editions/aglc4/meta.toml"),
    );
    pub const REPORTERS: (&str, &str) = (
        "data/editions/aglc4/tables/reporters.toml",
        include_str!("../../../data/editions/aglc4/tables/reporters.toml"),
    );
    pub const COURTS: (&str, &str) = (
        "data/editions/aglc4/tables/courts.toml",
        include_str!("../../../data/editions/aglc4/tables/courts.toml"),
    );
    pub const JURISDICTIONS: (&str, &str) = (
        "data/editions/aglc4/tables/jurisdictions.toml",
        include_str!("../../../data/editions/aglc4/tables/jurisdictions.toml"),
    );
    pub const CASE_RULES: (&str, &str) = (
        "data/editions/aglc4/rules/cases.ir.toml",
        include_str!("../../../data/editions/aglc4/rules/cases.ir.toml"),
    );
}

/// Edition metadata surfaced to consumers (subset of `meta.toml`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditionMeta {
    /// Edition identifier (`aglc4`).
    pub id: String,
    /// Human label (`Australian Guide to Legal Citation (4th ed)`).
    pub label: String,
    /// The guide's own preferred citation, for attribution.
    pub citation: String,
}

/// A loaded, validated, lookup-ready edition.
#[derive(Debug, Clone)]
pub struct EditionTables {
    /// Edition metadata.
    pub meta: EditionMeta,
    reporters: BTreeMap<String, Reporter>,
    courts: BTreeMap<String, Court>,
    jurisdictions: BTreeMap<String, Jurisdiction>,
    rules: Vec<RuleIr>,
}

impl EditionTables {
    /// Look up a reporter series by its exact abbreviation.
    pub fn reporter(&self, abbrev: &str) -> Option<&Reporter> {
        self.reporters.get(abbrev)
    }

    /// Look up a court by its medium-neutral identifier.
    pub fn court(&self, id: &str) -> Option<&Court> {
        self.courts.get(id)
    }

    /// Look up a jurisdiction by its abbreviation.
    pub fn jurisdiction(&self, abbrev: &str) -> Option<&Jurisdiction> {
        self.jurisdictions.get(abbrev)
    }

    /// All reporters, in deterministic (sorted) order.
    pub fn reporters(&self) -> impl Iterator<Item = &Reporter> {
        self.reporters.values()
    }

    /// All courts, in deterministic (sorted) order.
    pub fn courts(&self) -> impl Iterator<Item = &Court> {
        self.courts.values()
    }

    /// The declarative rule entries for this edition, in file order.
    pub fn rules(&self) -> &[RuleIr] {
        &self.rules
    }
}

/// Edition ids embedded in this build, in deterministic order.
pub fn editions() -> Vec<&'static str> {
    vec!["aglc4"]
}

/// Load and validate an edition by id. The default edition is `aglc4`.
pub fn load(edition_id: &str) -> Result<EditionTables, DataError> {
    match edition_id {
        "aglc4" => load_aglc4(),
        other => Err(DataError::UnknownEdition {
            requested: other.to_string(),
            available: editions().iter().map(|s| s.to_string()).collect(),
        }),
    }
}

fn load_aglc4() -> Result<EditionTables, DataError> {
    let meta = parse_meta(aglc4_files::META.0, aglc4_files::META.1)?;
    let reporters = tables::parse_reporters(aglc4_files::REPORTERS.0, aglc4_files::REPORTERS.1)?;
    let courts = tables::parse_courts(aglc4_files::COURTS.0, aglc4_files::COURTS.1)?;
    let jurisdictions =
        tables::parse_jurisdictions(aglc4_files::JURISDICTIONS.0, aglc4_files::JURISDICTIONS.1)?;
    let rules = ir::parse_rules(aglc4_files::CASE_RULES.0, aglc4_files::CASE_RULES.1)?;

    let mut reporter_map = BTreeMap::new();
    for r in reporters {
        if reporter_map.insert(r.abbrev.clone(), r).is_some() {
            return Err(DataError::Duplicate {
                file: aglc4_files::REPORTERS.0,
                id: "duplicate reporter abbreviation".to_string(),
            });
        }
    }
    let mut court_map = BTreeMap::new();
    for c in courts {
        let id = c.id.clone();
        if court_map.insert(id.clone(), c).is_some() {
            return Err(DataError::Duplicate {
                file: aglc4_files::COURTS.0,
                id,
            });
        }
    }
    let mut jur_map = BTreeMap::new();
    for j in jurisdictions {
        let id = j.abbrev.clone();
        if jur_map.insert(id.clone(), j).is_some() {
            return Err(DataError::Duplicate {
                file: aglc4_files::JURISDICTIONS.0,
                id,
            });
        }
    }

    // Cross-validate: reporter court/jurisdiction references must resolve
    // (no dangling vocab references — plan 01 T3).
    for r in reporter_map.values() {
        if let Some(court) = &r.court {
            if !court_map.contains_key(court) {
                return Err(DataError::DanglingReference {
                    rule_id: format!("reporter '{}'", r.abbrev),
                    message: format!("court '{court}' not in courts table"),
                });
            }
        }
        if let Some(j) = &r.jurisdiction {
            if !jur_map.contains_key(j) {
                return Err(DataError::DanglingReference {
                    rule_id: format!("reporter '{}'", r.abbrev),
                    message: format!("jurisdiction '{j}' not in table"),
                });
            }
        }
    }
    for c in court_map.values() {
        if !jur_map.contains_key(&c.jurisdiction) {
            return Err(DataError::DanglingReference {
                rule_id: format!("court '{}'", c.id),
                message: format!("jurisdiction '{}' not in table", c.jurisdiction),
            });
        }
    }

    Ok(EditionTables {
        meta,
        reporters: reporter_map,
        courts: court_map,
        jurisdictions: jur_map,
        rules,
    })
}

fn parse_meta(file: &'static str, src: &str) -> Result<EditionMeta, DataError> {
    let root = minitoml::parse(file, src)?;
    let edition = minitoml::req_table(file, "meta", &root, "edition")?;
    let citation = minitoml::req_table(file, "meta", &root, "citation")?;
    Ok(EditionMeta {
        id: minitoml::req_str(file, "edition", edition, "id")?.to_string(),
        label: minitoml::req_str(file, "edition", edition, "label")?.to_string(),
        citation: minitoml::req_str(file, "citation", citation, "text")?.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::YearBracket;

    #[test]
    fn load_aglc4_succeeds() {
        let t = load("aglc4").expect("aglc4 must load");
        assert_eq!(t.meta.id, "aglc4");
        assert!(t.meta.label.contains("4th ed"));
        assert!(!t.rules().is_empty());
    }

    #[test]
    fn unknown_edition_is_typed_error() {
        let err = load("aglc5").expect_err("aglc5 not embedded");
        match err {
            DataError::UnknownEdition { available, .. } => {
                assert_eq!(available, vec!["aglc4".to_string()]);
            }
            other => panic!("wrong error: {other}"),
        }
    }

    // Table-tests (plan 01 T5): known entries are asserted so a later
    // re-ingestion (plan 02) cannot silently regress them.

    #[test]
    fn table_test_clr_is_round_year_high_court() {
        let t = load("aglc4").expect("load");
        let clr = t.reporter("CLR").expect("CLR present");
        assert_eq!(clr.year_bracket, YearBracket::Round);
        assert_eq!(clr.court.as_deref(), Some("HCA"));
        assert_eq!(clr.jurisdiction.as_deref(), Some("Cth"));
        assert_eq!(clr.full, "Commonwealth Law Reports");
    }

    #[test]
    fn table_test_vr_is_square_year() {
        let t = load("aglc4").expect("load");
        let vr = t.reporter("VR").expect("VR present");
        assert_eq!(vr.year_bracket, YearBracket::Square);
    }

    #[test]
    fn table_test_hca_is_medium_neutral_cth() {
        let t = load("aglc4").expect("load");
        let hca = t.court("HCA").expect("HCA present");
        assert!(hca.medium_neutral);
        assert_eq!(hca.jurisdiction, "Cth");
        assert_eq!(hca.full, "High Court of Australia");
    }

    #[test]
    fn table_test_all_jurisdictions_present() {
        let t = load("aglc4").expect("load");
        for j in ["Cth", "NSW", "Vic", "Qld", "SA", "WA", "Tas", "ACT", "NT"] {
            assert!(t.jurisdiction(j).is_some(), "missing jurisdiction {j}");
        }
    }

    #[test]
    fn every_row_carries_provenance() {
        let t = load("aglc4").expect("load");
        for r in t.reporters() {
            assert!(!r.provenance.anchor.is_empty(), "{} anchor", r.abbrev);
            assert!(!r.provenance.reviewer.is_empty(), "{} reviewer", r.abbrev);
        }
        for c in t.courts() {
            assert!(!c.provenance.anchor.is_empty(), "{} anchor", c.id);
        }
        for rule in t.rules() {
            assert!(!rule.provenance.anchor.is_empty(), "{} anchor", rule.id);
        }
    }

    #[test]
    fn rule_codes_are_unique_and_well_formed() {
        let t = load("aglc4").expect("load");
        let mut seen = std::collections::BTreeSet::new();
        for rule in t.rules() {
            assert!(rule.id.starts_with("AGLC4-"), "{}", rule.id);
            assert!(seen.insert(rule.id.clone()), "duplicate {}", rule.id);
            assert!(!rule.aglc_ref.rule.is_empty(), "{} aglc_ref", rule.id);
        }
    }
}
