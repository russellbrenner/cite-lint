//! Disambiguation order (plan 03 T3; R-CORE-1).
//!
//! What: the single place a structurally-parsed citation body is classified
//! into a citation kind. The order is explicit, documented, and tested per
//! branch including the fall-through (CLAUDE.md correctness guardrail).
//! How: consults the edition tables through their public lookup API only.
//! Depends on: `lintcite-data` lookups; [`crate::ast`].
//!
//! Order for the M1 case slice:
//! 1. Body with a volume number → **Reported** (medium-neutral citations
//!    never carry a separate volume).
//! 2. Body without a volume, where the word run is a known **court id** →
//!    **MediumNeutral** (court ids are checked before reporters because the
//!    `[year] ID number` shape is the more specific match).
//! 3. Body without a volume, where the word run is a known **reporter** →
//!    **Reported** (year-organised series).
//! 4. Neither table knows the word run → **Reported** with an unknown
//!    series; the low-confidence `reporter-known` rule surfaces it. The
//!    engine never guesses beyond this — a wrong confident classification
//!    is worse than an honest unknown.
//!
//! Legislation (M3) and secondary sources (M5) insert between steps 3 and 4
//! additively; the fall-through then becomes `Unclassified`.

use lintcite_data::EditionTables;

use crate::ast::{Citation, CitationKind, MediumNeutral, PartyNames, Reported, Year};
use crate::parser::Body;

/// Classify a structurally-parsed body. See the module docs for the order.
pub(crate) fn classify(
    src: &str,
    parties: Option<PartyNames>,
    year: Year,
    body: Body,
    tables: &EditionTables,
) -> Citation {
    let raw = src.to_string();
    let series_key = body.series.normalised.as_str();

    // Step 1: a volume number means a volumed report series.
    if body.volume.is_some() {
        return Citation {
            raw,
            kind: CitationKind::Reported(Reported {
                parties,
                year,
                volume: body.volume,
                series: body.series,
                page: body.number,
                pinpoint: body.pinpoint,
            }),
        };
    }

    // Step 2: known court identifier → medium-neutral.
    if tables.court(series_key).is_some() {
        return Citation {
            raw,
            kind: CitationKind::MediumNeutral(MediumNeutral {
                parties,
                year,
                court_id: body.series,
                number: body.number,
            }),
        };
    }

    // Steps 3 and 4: reported, with the reporter either known or surfaced
    // by the low-confidence `reporter-known` rule.
    Citation {
        raw,
        kind: CitationKind::Reported(Reported {
            parties,
            year,
            volume: None,
            series: body.series,
            page: body.number,
            pinpoint: body.pinpoint,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use lintcite_data::load;

    fn tables() -> EditionTables {
        load("aglc4").expect("embedded aglc4 edition loads")
    }

    #[test]
    fn branch_1_volume_forces_reported() {
        let t = tables();
        // `HCA` is a court id, but the volume makes this Reported.
        let c = parse("X v Y (1992) 12 HCA 1", &t);
        assert!(
            matches!(c.kind, CitationKind::Reported(_)),
            "volume must force Reported: {:?}",
            c.kind
        );
    }

    #[test]
    fn branch_2_court_id_is_medium_neutral() {
        let t = tables();
        let c = parse("X v Y [2020] FCAFC 12", &t);
        assert!(matches!(c.kind, CitationKind::MediumNeutral(_)));
    }

    #[test]
    fn branch_3_known_reporter_is_reported() {
        let t = tables();
        let c = parse("X v Y [1969] VR 403", &t);
        let CitationKind::Reported(r) = &c.kind else {
            panic!("expected Reported, got {:?}", c.kind);
        };
        assert_eq!(r.volume, None);
        assert_eq!(r.series.normalised, "VR");
    }

    #[test]
    fn branch_4_unknown_series_falls_through_to_reported_unknown() {
        let t = tables();
        let c = parse("X v Y [2020] ZZZQ 12", &t);
        // Falls through as Reported with an unknown series; the
        // low-confidence reporter-known rule (AGLC4-CASE-005) surfaces it.
        let CitationKind::Reported(r) = &c.kind else {
            panic!("expected fall-through Reported, got {:?}", c.kind);
        };
        assert!(t.reporter(&r.series.normalised).is_none());
        assert!(t.court(&r.series.normalised).is_none());
    }
}
