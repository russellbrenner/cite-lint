# cite-lint rule catalogue (aglc4)

> Generated from the rule registry (P6: docs-from-code). Do not edit by hand — the `rule_catalogue_is_current` test regenerates this content and fails CI when it drifts.

## AGLC4-CASE-001 — AGLC4 r 2.2.1

- severity: error · confidence: high · fix-it: swap-year-bracket
- anchor: Part 2 - Cases: year and volume
- provenance: AGLC4 r 2.2.1 (Year and Volume): volumed series take round brackets; year-organised series take square brackets

year bracket should be {expected} for {reporter}: found {found}

## AGLC4-CASE-002 — AGLC4 r 2.1.11

- severity: error · confidence: high · fix-it: normalise-party-separator
- anchor: Part 2 - Cases: party separator
- provenance: AGLC4 r 2.1.11: parties separated by 'v' (general) / '&' (family law); no full stop per r 1.6.1

parties should be separated by an unpunctuated lowercase 'v': found '{found}'

## AGLC4-CASE-003 — AGLC4 r 1.6.1

- severity: error · confidence: high · fix-it: strip-reporter-dots
- anchor: Part 1 - General Rules: full stops in abbreviations
- provenance: AGLC4 r 1.6.1 (p 22): full stops are not used in abbreviations or after initials

report series abbreviations take no full stops: found '{found}', expected '{expected}'

## AGLC4-CASE-004 — AGLC4 r 2.2.5

- severity: warning · confidence: high · fix-it: normalise-pinpoint
- anchor: Part 2 - Cases: pinpoint references
- provenance: AGLC4 r 2.2.5 (reported-case pinpoint); comma + space before pinpoint per r 1.1.6-1.1.7

pinpoint should follow the starting page as ', <pinpoint>': found '{found}'

## AGLC4-CASE-005 — AGLC4 r 2.2.3

- severity: info · confidence: low · fix-it: none
- anchor: Part 2 - Cases: law report series (Appendix A)
- provenance: AGLC4 Appendix A: recognised law report series

could not verify report series '{found}' against the AGLC4 vocabulary

## AGLC4-CASE-006 — AGLC4 r 2.3.1

- severity: error · confidence: high · fix-it: square-year-bracket
- anchor: Part 2 - Cases: medium neutral citations
- provenance: AGLC4 r 2.3.1: [year] CourtId Number

medium-neutral citations take square brackets around the year: found {found}

## AGLC4-CASE-007 — AGLC4 r 2.2.1

- severity: error · confidence: high · fix-it: none
- anchor: Part 2 - Cases: year and volume
- provenance: AGLC4 r 2.2.1 (Year and Volume): volumed series cite (year) volume series page

{reporter} is organised by volume: a volume number must precede the series abbreviation

