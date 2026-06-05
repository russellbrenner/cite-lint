//! SDK facade integration tests: parity harness (plan 00 T5 / 07 T7),
//! determinism (P3), fix-it safety (plan 03 T6), and the no-panic sweep
//! (plan 07 T3, dependency-free property testing).

use cite_lint::{Confidence, HostKind, Session, CAPABILITIES};

fn session() -> Session {
    Session::new("aglc4").expect("default edition loads")
}

// ---------------------------------------------------------------------------
// Parity harness: every declared capability is reachable from the SDK.
// Surface reachability (CLI) is asserted in cite-lint-cli's e2e tests; the
// LSP/MCP columns activate at their milestones (plan 06).
// ---------------------------------------------------------------------------

#[test]
fn parity_every_capability_is_reachable_from_the_sdk() {
    let s = session();
    for capability in CAPABILITIES {
        match *capability {
            "lint" => {
                let _ = s.lint("x", HostKind::Plain);
            }
            "parse" => {
                let _ = s.parse_citation("x");
            }
            "explain" => {
                assert!(s.explain("AGLC4-CASE-001").is_some());
            }
            "fix" => {
                let _ = s.fix("x", HostKind::Plain);
            }
            "tokens" => {
                // Typed Unimplemented until the LSP slice (plan 00 T4).
                assert!(s.tokens().is_err());
            }
            "editions" => {
                let editions = s.editions().expect("editions list");
                assert!(editions.iter().any(|e| e.id == "aglc4"));
            }
            other => panic!("capability '{other}' has no SDK mapping"),
        }
    }
}

#[test]
fn parity_capability_set_is_the_documented_six() {
    assert_eq!(
        CAPABILITIES,
        &["lint", "parse", "explain", "fix", "tokens", "editions"]
    );
}

// ---------------------------------------------------------------------------
// Determinism (P3): same (input, edition) ⇒ identical diagnostics.
// ---------------------------------------------------------------------------

#[test]
fn determinism_repeated_runs_are_identical() {
    let s = session();
    let doc = "[^1]: Mabo v. Queensland [1992] 175 C.L.R. 1, 30.\n[^2]: Love v Commonwealth (2020) HCA 3.\n";
    let a = format!("{:?}", s.lint(doc, HostKind::Markdown));
    for _ in 0..10 {
        let b = format!("{:?}", s.lint(doc, HostKind::Markdown));
        assert_eq!(a, b, "diagnostics must be byte-identical across runs");
    }
    // And across fresh sessions (no hidden global state).
    let s2 = session();
    assert_eq!(a, format!("{:?}", s2.lint(doc, HostKind::Markdown)));
}

// ---------------------------------------------------------------------------
// Fix-it safety: idempotent, meaning-preserving (plan 03 T6).
// ---------------------------------------------------------------------------

#[test]
fn fix_makes_the_document_pass_and_is_idempotent() {
    let s = session();
    let doc = "[^1]: Mabo v. Queensland [1992] 175 C.L.R. 1.\n";
    let first = s.fix(doc, HostKind::Markdown);
    assert!(first.applied >= 3, "expected separator+bracket+dots fixes");
    assert_eq!(
        first.fixed, "[^1]: Mabo v Queensland (1992) 175 CLR 1.\n",
        "fixes compose into the compliant citation"
    );
    // Re-linting the fixed text yields no fixable diagnostics.
    let residual = s.lint(&first.fixed, HostKind::Markdown);
    assert!(
        residual.iter().all(|d| d.fix.is_none()),
        "no fixable diagnostics may remain: {residual:?}"
    );
    // Idempotence: fixing again changes nothing.
    let second = s.fix(&first.fixed, HostKind::Markdown);
    assert_eq!(second.applied, 0);
    assert_eq!(second.fixed, first.fixed);
}

#[test]
fn fix_never_touches_volume_page_or_authority() {
    let s = session();
    let doc = "[^1]: Mabo v. Queensland [1992] 175 C.L.R. 1, 30.\n";
    let fixed = s.fix(doc, HostKind::Markdown).fixed;
    for preserved in ["Mabo", "Queensland", "175", "1, 30"] {
        assert!(
            fixed.contains(preserved),
            "fix altered protected content '{preserved}': {fixed}"
        );
    }
}

// ---------------------------------------------------------------------------
// Low-confidence guardrail: unknown vocabulary is surfaced, never guessed.
// ---------------------------------------------------------------------------

#[test]
fn unknown_reporter_is_low_confidence_not_confident_wrong() {
    let s = session();
    let diags = s.lint("X v Y (1992) 12 ZZQ 99\n", HostKind::Plain);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.0, "AGLC4-CASE-005");
    assert_eq!(diags[0].confidence, Confidence::Low);
    assert!(diags[0].fix.is_none());
}

// ---------------------------------------------------------------------------
// No-panic sweep (plan 07 T3a, dependency-free): deterministic LCG mutations
// of seed citations must never panic anywhere in the pipeline.
// ---------------------------------------------------------------------------

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        // Numerical Recipes LCG constants; determinism is the point.
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
}

#[test]
fn pipeline_never_panics_on_mutated_inputs() {
    let s = session();
    let seeds = [
        "Mabo v Queensland (No 2) (1992) 175 CLR 1",
        "[^1]: Love v Commonwealth [2020] HCA 3.",
        "Plaintiff S157/2002 v Commonwealth (2003) 211 CLR 476, 492",
        "X v Y [1969] VR 403; Z v W (1992) 175 CLR 1",
        "(1992) 175 CLR",
        "[[[(((1992)))]]]",
    ];
    let mut rng = Lcg(0x5DEECE66D);
    for seed in seeds {
        for _ in 0..200 {
            let mut bytes = seed.as_bytes().to_vec();
            // Up to 4 mutations: overwrite, insert, delete.
            for _ in 0..(rng.next() % 4 + 1) {
                if bytes.is_empty() {
                    break;
                }
                let pos = (rng.next() as usize) % bytes.len();
                match rng.next() % 3 {
                    0 => bytes[pos] = (rng.next() % 256) as u8,
                    1 => bytes.insert(pos, (rng.next() % 128) as u8),
                    _ => {
                        bytes.remove(pos);
                    }
                }
            }
            let input = String::from_utf8_lossy(&bytes).into_owned();
            // Whole pipeline: extraction, parsing, linting, fixing.
            let _ = s.lint(&input, HostKind::Markdown);
            let _ = s.lint(&input, HostKind::Plain);
            let _ = s.fix(&input, HostKind::Plain);
            let _ = s.parse_citation(&input);
        }
    }
}
