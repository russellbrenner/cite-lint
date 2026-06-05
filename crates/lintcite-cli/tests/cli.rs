//! CLI end-to-end tests (plan 07 T6): real binary, fixture corpus, exit
//! codes, both output formats. This is also the CLI column of the parity
//! matrix — every capability is exercised through the binary.

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lintcite"))
}

fn fixture(name: &str) -> String {
    format!("{}/../../testdata/e2e/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn check_flags_the_memo_fixture_and_exits_1() {
    let out = bin()
        .args(["check", &fixture("memo.md")])
        .output()
        .expect("binary runs");
    assert_eq!(out.status.code(), Some(1), "diagnostics → exit 1");
    let text = stdout(&out);
    for expected in [
        "AGLC4-CASE-001", // [1992] 175 CLR — square on a round series
        "AGLC4-CASE-002", // v.
        "AGLC4-CASE-003", // F.C.R.
        "AGLC4-CASE-006", // (2020) HCA 3
    ] {
        assert!(text.contains(expected), "missing {expected} in:\n{text}");
    }
    assert!(
        !text.contains("AGLC4-CASE-005"),
        "all fixture reporters are known:\n{text}"
    );
    // Footnote 4 is compliant and must produce nothing.
    assert!(
        !text.contains("S157"),
        "compliant citation flagged:\n{text}"
    );
    // Messages name their AGLC4 rule (correctness guardrail).
    assert!(text.contains("AGLC4 r 2.2.1"), "{text}");
}

#[test]
fn check_clean_input_exits_0_with_no_output() {
    let out = bin()
        .args(["check", "--host", "plain", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child
                .stdin
                .take()
                .expect("stdin")
                .write_all(b"Mabo v Queensland (No 2) (1992) 175 CLR 1\n")?;
            child.wait_with_output()
        })
        .expect("binary runs");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
}

#[test]
fn check_json_emits_the_versioned_schema() {
    let out = bin()
        .args(["check", "--format", "json", &fixture("memo.md")])
        .output()
        .expect("binary runs");
    assert_eq!(out.status.code(), Some(1));
    let json = stdout(&out);
    assert!(
        json.starts_with("{\"schema_version\":1,\"edition\":\"aglc4\""),
        "{json}"
    );
    assert!(json.contains("\"code\":\"AGLC4-CASE-001\""), "{json}");
    assert!(json.contains("\"aglc_rule\":\"2.2.1\""), "{json}");
    assert!(json.trim_end().ends_with('}'), "{json}");
}

#[test]
fn check_output_is_deterministic_across_runs() {
    let run = || {
        stdout(
            &bin()
                .args(["check", "--format", "json", &fixture("memo.md")])
                .output()
                .expect("binary runs"),
        )
    };
    let first = run();
    for _ in 0..3 {
        assert_eq!(run(), first, "byte-identical output (P3)");
    }
}

#[test]
fn parse_renders_a_typed_ast() {
    let out = bin()
        .args(["parse", "Mabo v Queensland (No 2) (1992) 175 CLR 1"])
        .output()
        .expect("binary runs");
    assert_eq!(out.status.code(), Some(0));
    let text = stdout(&out);
    assert!(text.contains("Reported"), "{text}");
    assert!(text.contains("CLR"), "{text}");
}

#[test]
fn explain_shows_rule_metadata_and_provenance() {
    let out = bin()
        .args(["explain", "AGLC4-CASE-001"])
        .output()
        .expect("binary runs");
    assert_eq!(out.status.code(), Some(0));
    let text = stdout(&out);
    assert!(text.contains("2.2.1"), "{text}");
    assert!(text.contains("provenance:"), "{text}");
}

#[test]
fn explain_all_lists_every_rule() {
    let out = bin().args(["explain", "--all"]).output().expect("runs");
    let text = stdout(&out);
    for code in [
        "AGLC4-CASE-001",
        "AGLC4-CASE-002",
        "AGLC4-CASE-003",
        "AGLC4-CASE-004",
        "AGLC4-CASE-005",
        "AGLC4-CASE-006",
        "AGLC4-CASE-007",
    ] {
        assert!(text.contains(code), "missing {code}:\n{text}");
    }
}

#[test]
fn fix_outputs_the_corrected_document() {
    let out = bin()
        .args(["fix", &fixture("memo.md")])
        .output()
        .expect("binary runs");
    assert_eq!(out.status.code(), Some(0));
    let text = stdout(&out);
    assert!(
        text.contains("Mabo v Queensland (No 2) (1992) 175 CLR 1."),
        "{text}"
    );
    assert!(
        text.contains("Wik Peoples v Queensland (1996) 187 CLR 1."),
        "{text}"
    );
    assert!(text.contains("(2001) 110 FCR 491"), "{text}");
    assert!(text.contains("Love v Commonwealth [2020] HCA 3."), "{text}");
    // Authority content is untouched.
    assert!(text.contains("211 CLR 476, 492"), "{text}");
}

#[test]
fn editions_lists_aglc4_with_attribution() {
    let out = bin().args(["editions"]).output().expect("binary runs");
    assert_eq!(out.status.code(), Some(0));
    let text = stdout(&out);
    assert!(text.contains("aglc4"), "{text}");
    assert!(
        text.contains("Australian Guide to Legal Citation"),
        "{text}"
    );
}

#[test]
fn unknown_subcommand_exits_2() {
    let out = bin().args(["frobnicate"]).output().expect("binary runs");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn missing_file_exits_2() {
    let out = bin()
        .args(["check", "/nonexistent/nope.md"])
        .output()
        .expect("binary runs");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn help_lists_every_capability_subcommand() {
    let out = bin().args(["--help"]).output().expect("binary runs");
    let text = stdout(&out);
    for sub in ["check", "parse", "explain", "fix", "editions"] {
        assert!(text.contains(sub), "help missing '{sub}':\n{text}");
    }
}
