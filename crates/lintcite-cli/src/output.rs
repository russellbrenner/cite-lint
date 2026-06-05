//! CLI output formatting: human text and the versioned JSON schema
//! (plan 06 T2). Pure functions over the SDK's `Diagnostic` model —
//! the CLI translates, it never invents (P8).

use lintcite::{Confidence, Diagnostic, Severity};

/// Render one diagnostic as a human line (plus an optional fix line):
/// `path:line:col: severity[CODE] message`.
pub fn text_line(path: &str, source: &str, d: &Diagnostic) -> String {
    let (line, col) = line_col(source, d.range.start);
    let severity = severity_label(d.severity);
    let mut out = format!("{path}:{line}:{col}: {severity}[{}] {}", d.code, d.message);
    if let Some(fix) = &d.fix {
        out.push_str(&format!("\n    fix-it: '{}'", fix.replacement));
    }
    out
}

/// 1-based (line, byte-column) of a byte offset in `source`.
fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let clamped = offset.min(source.len());
    let mut line = 1usize;
    let mut line_start = 0usize;
    for (i, b) in source.bytes().enumerate().take(clamped) {
        if b == b'\n' {
            line += 1;
            line_start = i + 1;
        }
    }
    (line, clamped - line_start + 1)
}

/// Lowercase severity label, stable across formats.
pub fn severity_label(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Hint => "hint",
    }
}

fn confidence_label(c: Confidence) -> &'static str {
    match c {
        Confidence::High => "high",
        Confidence::Low => "low",
    }
}

/// Escape a string for JSON output.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Render all diagnostics for a run as the versioned JSON document
/// (schema_version 1; additions are backwards-compatible, field order is
/// stable for the determinism gate).
pub fn json_document(edition: &str, per_file: &[(String, Vec<Diagnostic>)]) -> String {
    let mut items = Vec::new();
    for (path, diags) in per_file {
        for d in diags {
            let fix = match &d.fix {
                Some(f) => format!(
                    "{{\"range\":{{\"start\":{},\"end\":{}}},\"replacement\":\"{}\"}}",
                    f.range.start,
                    f.range.end,
                    json_escape(&f.replacement)
                ),
                None => "null".to_string(),
            };
            items.push(format!(
                "{{\"code\":\"{}\",\"message\":\"{}\",\"severity\":\"{}\",\"confidence\":\"{}\",\"path\":\"{}\",\"range\":{{\"start\":{},\"end\":{}}},\"aglc_rule\":\"{}\",\"fix\":{}}}",
                json_escape(&d.code.0),
                json_escape(&d.message),
                severity_label(d.severity),
                confidence_label(d.confidence),
                json_escape(path),
                d.range.start,
                d.range.end,
                json_escape(&d.rule_ref.aglc.rule),
                fix
            ));
        }
    }
    format!(
        "{{\"schema_version\":1,\"edition\":\"{}\",\"diagnostics\":[{}]}}",
        json_escape(edition),
        items.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_col_is_one_based_and_newline_aware() {
        let src = "abc\ndef\n";
        assert_eq!(line_col(src, 0), (1, 1));
        assert_eq!(line_col(src, 4), (2, 1));
        assert_eq!(line_col(src, 6), (2, 3));
    }

    #[test]
    fn json_escape_handles_quotes_and_controls() {
        assert_eq!(json_escape("a\"b\\c\nd"), "a\\\"b\\\\c\\nd");
        assert_eq!(json_escape("\u{1}"), "\\u0001");
    }

    #[test]
    fn empty_run_is_a_valid_document() {
        let doc = json_document("aglc4", &[]);
        assert_eq!(
            doc,
            "{\"schema_version\":1,\"edition\":\"aglc4\",\"diagnostics\":[]}"
        );
    }
}
