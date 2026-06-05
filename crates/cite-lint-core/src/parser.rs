//! Recursive-descent case-citation parser with error recovery (plan 03 T2).
//!
//! What: citation string → typed, classified [`Citation`]. Malformed input
//! yields `CitationKind::Unclassified` with a reason — a partial result,
//! never a panic (property-tested in `tests/`).
//! How: lex, locate the year element, split parties / body around it, match
//! the body against the reported and medium-neutral shapes, then classify
//! via [`crate::classify`] (which consults the vocabulary tables).
//! Depends on: [`crate::lexer`], [`crate::ast`], [`crate::classify`],
//! `cite-lint-data` tables.
//!
//! Parser strategy: hand-written recursive descent, the branch of invariant 4
//! ("hand-written / PEG (chumsky)") chosen while R-ARCH-1 (chumsky error
//! recovery) remains open — see docs/adr/0001-m1-provisional-choices.md.

use cite_lint_data::{EditionTables, YearBracket};

use crate::ast::{Citation, CitationKind, PartyNames, Pinpoint, Separator, Series, Year};
use crate::classify;
use crate::diagnostic::SourceRange;
use crate::lexer::{lex, Token, TokenKind};

/// Recognised party separators, exactly as written.
const SEPARATORS: &[&str] = &["v", "v.", "vs", "vs.", "V", "V.", "Vs", "Vs."];

/// Parse and classify one citation string.
///
/// `tables` is consulted only for classification (court vs reporter); the
/// structural parse is table-free. Always returns a [`Citation`]; never
/// panics on any input.
pub fn parse(src: &str, tables: &EditionTables) -> Citation {
    let raw = src.to_string();
    let mut tokens = lex(src);
    strip_trailing_punctuation(&mut tokens);

    let Some(year_idx) = find_year(&tokens) else {
        return Citation {
            raw,
            kind: CitationKind::Unclassified {
                reason: "no year element (a bracketed four-digit year) found".to_string(),
            },
        };
    };

    let year = year_from(&tokens, year_idx);
    let parties = parse_parties(src, &tokens[..year_idx]);
    let body = &tokens[year_idx + 3..];

    match parse_body(src, body) {
        Ok(shape) => classify::classify(src, parties, year, shape, tables),
        Err(reason) => Citation {
            raw,
            kind: CitationKind::Unclassified { reason },
        },
    }
}

/// The structural shape of the post-year body, before classification.
pub(crate) struct Body {
    /// Leading volume number, when present.
    pub volume: Option<u32>,
    /// The series / court-identifier word run.
    pub series: Series,
    /// The trailing number (page, or judgment number for medium-neutral).
    pub number: u32,
    /// Anything after the trailing number (pinpoint candidate).
    pub pinpoint: Option<Pinpoint>,
}

/// Drop trailing `.`/`;` punctuation tokens (footnote sentence endings).
fn strip_trailing_punctuation(tokens: &mut Vec<Token<'_>>) {
    while let Some(last) = tokens.last() {
        let droppable = matches!(
            last.kind,
            TokenKind::Other | TokenKind::Semicolon | TokenKind::Comma
        );
        if droppable {
            tokens.pop();
        } else {
            break;
        }
    }
}

/// Find the year element: `(` Int `)` or `[` Int `]` with a plausible
/// four-digit year. Returns the index of the opening bracket token.
fn find_year(tokens: &[Token<'_>]) -> Option<usize> {
    for i in 0..tokens.len().saturating_sub(2) {
        let (open, num, close) = (&tokens[i], &tokens[i + 1], &tokens[i + 2]);
        let pair_ok = matches!(
            (open.kind, close.kind),
            (TokenKind::OpenParen, TokenKind::CloseParen)
                | (TokenKind::OpenSquare, TokenKind::CloseSquare)
        );
        if !pair_ok || num.kind != TokenKind::Int || num.text.len() != 4 {
            continue;
        }
        if let Ok(value) = num.text.parse::<u32>() {
            if (1500..=2099).contains(&value) {
                return Some(i);
            }
        }
    }
    None
}

fn year_from(tokens: &[Token<'_>], idx: usize) -> Year {
    let open = &tokens[idx];
    let num = &tokens[idx + 1];
    let close = &tokens[idx + 2];
    let bracket = match open.kind {
        TokenKind::OpenParen => YearBracket::Round,
        _ => YearBracket::Square,
    };
    Year {
        // find_year validated the parse; default keeps this total.
        value: num.text.parse().unwrap_or(0),
        bracket,
        range: SourceRange::new(open.start, close.end),
    }
}

/// Parties: everything before the year element, with separator detection.
fn parse_parties(src: &str, tokens: &[Token<'_>]) -> Option<PartyNames> {
    let (first, last) = (tokens.first()?, tokens.last()?);
    let range = SourceRange::new(first.start, last.end);
    let raw = src[range.start..range.end].to_string();

    let mut depth = 0usize;
    let mut separator = None;
    for t in tokens {
        match t.kind {
            TokenKind::OpenParen | TokenKind::OpenSquare => depth += 1,
            TokenKind::CloseParen | TokenKind::CloseSquare => depth = depth.saturating_sub(1),
            TokenKind::Word
                if depth == 0 && separator.is_none() && SEPARATORS.contains(&t.text) =>
            {
                separator = Some(Separator {
                    raw: t.text.to_string(),
                    range: SourceRange::new(t.start, t.end),
                });
            }
            _ => {}
        }
    }
    Some(PartyNames {
        raw,
        range,
        separator,
    })
}

/// Match the post-year token run against `[Int] Word+ Int [pinpoint...]`.
fn parse_body(src: &str, tokens: &[Token<'_>]) -> Result<Body, String> {
    let mut i = 0;

    // Optional leading volume.
    let volume = if matches!(tokens.first().map(|t| t.kind), Some(TokenKind::Int)) {
        let v = tokens[0]
            .text
            .parse::<u32>()
            .map_err(|_| "volume number out of range".to_string())?;
        i = 1;
        Some(v)
    } else {
        None
    };

    // Series: one or more Word tokens.
    let series_start = i;
    while i < tokens.len() && tokens[i].kind == TokenKind::Word {
        i += 1;
    }
    if i == series_start {
        return Err("expected a report series or court identifier after the year".to_string());
    }
    let s_first = &tokens[series_start];
    let s_last = &tokens[i - 1];
    let series_range = SourceRange::new(s_first.start, s_last.end);
    let series_raw = src[series_range.start..series_range.end].to_string();
    let normalised = normalise_series(&tokens[series_start..i]);
    let series = Series {
        raw: series_raw,
        normalised,
        range: series_range,
    };

    // Trailing number (page / judgment number).
    if i >= tokens.len() || tokens[i].kind != TokenKind::Int {
        return Err(format!(
            "expected a page or judgment number after '{}'",
            series.raw
        ));
    }
    let number = tokens[i]
        .text
        .parse::<u32>()
        .map_err(|_| "page number out of range".to_string())?;
    let number_end = tokens[i].end;
    i += 1;

    // Pinpoint candidate: any remaining source text.
    let pinpoint = if i < tokens.len() {
        let last = tokens.last().map_or(number_end, |t| t.end);
        let range = SourceRange::new(number_end, last);
        let raw = src[range.start..range.end].to_string();
        let well_formed = raw.starts_with(", ");
        Some(Pinpoint {
            raw,
            range,
            well_formed,
        })
    } else {
        None
    };

    Ok(Body {
        volume,
        series,
        number,
        pinpoint,
    })
}

/// Normalise a series word run: strip full stops, join with single spaces.
/// `C.L.R.` → `CLR`; `Fam LR` → `Fam LR`; `Qd R` → `Qd R`.
fn normalise_series(tokens: &[Token<'_>]) -> String {
    let mut parts = Vec::with_capacity(tokens.len());
    for t in tokens {
        let cleaned: String = t.text.chars().filter(|c| *c != '.').collect();
        if !cleaned.is_empty() {
            parts.push(cleaned);
        }
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use cite_lint_data::load;

    fn tables() -> EditionTables {
        load("aglc4").expect("embedded aglc4 edition loads")
    }

    // --- Golden ASTs: well-formed inputs (plan 07 T2). ---

    #[test]
    fn golden_reported_with_parties_and_volume() {
        let t = tables();
        let c = parse("Mabo v Queensland (No 2) (1992) 175 CLR 1", &t);
        let CitationKind::Reported(r) = &c.kind else {
            panic!("expected Reported, got {:?}", c.kind);
        };
        let p = r.parties.as_ref().expect("parties");
        assert_eq!(p.raw, "Mabo v Queensland (No 2)");
        assert_eq!(p.separator.as_ref().map(|s| s.raw.as_str()), Some("v"));
        assert_eq!(r.year.value, 1992);
        assert_eq!(r.year.bracket, YearBracket::Round);
        assert_eq!(r.volume, Some(175));
        assert_eq!(r.series.normalised, "CLR");
        assert_eq!(r.page, 1);
        assert!(r.pinpoint.is_none());
    }

    #[test]
    fn golden_reported_with_pinpoint() {
        let t = tables();
        let c = parse(
            "Plaintiff S157/2002 v Commonwealth (2003) 211 CLR 476, 492",
            &t,
        );
        let CitationKind::Reported(r) = &c.kind else {
            panic!("expected Reported, got {:?}", c.kind);
        };
        assert_eq!(r.page, 476);
        let pin = r.pinpoint.as_ref().expect("pinpoint");
        assert_eq!(pin.raw, ", 492");
        assert!(pin.well_formed);
    }

    #[test]
    fn golden_square_year_reported() {
        let t = tables();
        let c = parse("Donoghue v Stevenson [1932] AC 562", &t);
        let CitationKind::Reported(r) = &c.kind else {
            panic!("expected Reported, got {:?}", c.kind);
        };
        assert_eq!(r.year.bracket, YearBracket::Square);
        assert_eq!(r.volume, None);
        assert_eq!(r.series.normalised, "AC");
        assert_eq!(r.page, 562);
    }

    #[test]
    fn golden_medium_neutral() {
        let t = tables();
        let c = parse("Love v Commonwealth [2020] HCA 3", &t);
        let CitationKind::MediumNeutral(m) = &c.kind else {
            panic!("expected MediumNeutral, got {:?}", c.kind);
        };
        assert_eq!(m.year.value, 2020);
        assert_eq!(m.year.bracket, YearBracket::Square);
        assert_eq!(m.court_id.normalised, "HCA");
        assert_eq!(m.number, 3);
    }

    #[test]
    fn golden_multiword_series() {
        let t = tables();
        let c = parse("Smith v Smith (1996) 20 Fam LR 1", &t);
        let CitationKind::Reported(r) = &c.kind else {
            panic!("expected Reported, got {:?}", c.kind);
        };
        assert_eq!(r.series.normalised, "Fam LR");
    }

    #[test]
    fn golden_dotted_series_normalises() {
        let t = tables();
        let c = parse("Mabo v Queensland (1992) 175 C.L.R. 1", &t);
        let CitationKind::Reported(r) = &c.kind else {
            panic!("expected Reported, got {:?}", c.kind);
        };
        assert_eq!(r.series.raw, "C.L.R.");
        assert_eq!(r.series.normalised, "CLR");
    }

    #[test]
    fn trailing_full_stop_is_tolerated() {
        let t = tables();
        let c = parse("Donoghue v Stevenson [1932] AC 562.", &t);
        assert!(c.is_case(), "got {:?}", c.kind);
    }

    // --- Golden partial ASTs: malformed inputs recover (plan 07 T2). ---

    #[test]
    fn malformed_no_year_recovers_with_reason() {
        let t = tables();
        let c = parse("Mabo v Queensland 175 CLR 1", &t);
        let CitationKind::Unclassified { reason } = &c.kind else {
            panic!("expected Unclassified, got {:?}", c.kind);
        };
        assert!(reason.contains("year"), "{reason}");
    }

    #[test]
    fn malformed_missing_page_recovers_with_reason() {
        let t = tables();
        let c = parse("Mabo v Queensland (1992) 175 CLR", &t);
        let CitationKind::Unclassified { reason } = &c.kind else {
            panic!("expected Unclassified, got {:?}", c.kind);
        };
        assert!(reason.contains("page"), "{reason}");
    }

    #[test]
    fn malformed_missing_series_recovers_with_reason() {
        let t = tables();
        let c = parse("Mabo v Queensland (1992) 175", &t);
        assert!(matches!(c.kind, CitationKind::Unclassified { .. }));
    }

    #[test]
    fn empty_input_is_unclassified() {
        let t = tables();
        assert!(matches!(
            parse("", &t).kind,
            CitationKind::Unclassified { .. }
        ));
    }

    #[test]
    fn separator_variants_are_captured() {
        let t = tables();
        for (input, sep) in [
            ("Mabo v. Queensland (1992) 175 CLR 1", "v."),
            ("Mabo vs Queensland (1992) 175 CLR 1", "vs"),
            ("Mabo V Queensland (1992) 175 CLR 1", "V"),
        ] {
            let c = parse(input, &t);
            let p = c.parties().expect("parties");
            assert_eq!(
                p.separator.as_ref().map(|s| s.raw.as_str()),
                Some(sep),
                "input: {input}"
            );
        }
    }

    #[test]
    fn re_style_case_has_no_separator() {
        let t = tables();
        let c = parse("Re Wakim (1999) 198 CLR 511", &t);
        let p = c.parties().expect("parties");
        assert!(p.separator.is_none());
    }

    #[test]
    fn parenthesised_no_2_does_not_become_year() {
        let t = tables();
        let c = parse("Mabo v Queensland (No 2) (1992) 175 CLR 1", &t);
        let CitationKind::Reported(r) = &c.kind else {
            panic!("expected Reported");
        };
        assert_eq!(r.year.value, 1992);
    }

    #[test]
    fn ranges_map_back_into_source() {
        let t = tables();
        let src = "Mabo v Queensland (No 2) (1992) 175 CLR 1";
        let c = parse(src, &t);
        let CitationKind::Reported(r) = &c.kind else {
            panic!("expected Reported");
        };
        assert_eq!(&src[r.year.range.start..r.year.range.end], "(1992)");
        assert_eq!(&src[r.series.range.start..r.series.range.end], "CLR");
        let p = r.parties.as_ref().expect("parties");
        let s = p.separator.as_ref().expect("separator");
        assert_eq!(&src[s.range.start..s.range.end], "v");
    }
}
