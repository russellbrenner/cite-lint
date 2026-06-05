//! `lintcite` — the batch CLI surface (plan 06).
//!
//! A thin shell over the `lintcite` SDK: parse args → call a capability →
//! format output. No linting logic lives here (P1). Subcommands map 1:1 to
//! the SDK capability set; exit codes are stable for CI: `0` clean, `1`
//! diagnostics found, `2` usage or load error.
//!
//! Argument parsing is hand-rolled while the dependency policy holds
//! (R-CLI-1 / docs/adr/0001-m1-provisional-choices.md); the surface is
//! deliberately tiny so a later `clap` swap is mechanical.

mod output;

use std::io::Read;
use std::process::ExitCode;

use lintcite::{Diagnostic, HostKind, Session};

const USAGE: &str = "lintcite — AGLC4 citation linter

USAGE:
    lintcite check [PATH ...|-] [--format text|json] [--edition ID] [--host markdown|plain]
    lintcite parse <CITATION>
    lintcite explain <CODE>|--all
    lintcite fix <PATH> [--edition ID] [--host markdown|plain]
    lintcite editions
    lintcite --help | --version

EXIT CODES:
    0  no diagnostics
    1  diagnostics found (check)
    2  usage, IO, or edition-load error";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("lintcite: {message}");
            ExitCode::from(2)
        }
    }
}

/// Shared flag set for `check` and `fix`.
struct Flags {
    format: String,
    edition: String,
    host: Option<HostKind>,
    positional: Vec<String>,
}

fn parse_flags(args: &[String]) -> Result<Flags, String> {
    let mut flags = Flags {
        format: "text".to_string(),
        edition: "aglc4".to_string(),
        host: None,
        positional: Vec::new(),
    };
    let mut i = 0;
    while i < args.len() {
        let take_value = |i: &mut usize| -> Result<String, String> {
            *i += 1;
            args.get(*i)
                .cloned()
                .ok_or_else(|| format!("flag '{}' needs a value", args[*i - 1]))
        };
        match args[i].as_str() {
            "--format" => {
                flags.format = take_value(&mut i)?;
                if flags.format != "text" && flags.format != "json" {
                    return Err(format!(
                        "unknown format '{}' (expected text|json)",
                        flags.format
                    ));
                }
            }
            "--edition" => flags.edition = take_value(&mut i)?,
            "--host" => {
                flags.host = Some(match take_value(&mut i)?.as_str() {
                    "markdown" => HostKind::Markdown,
                    "plain" => HostKind::Plain,
                    other => {
                        return Err(format!("unknown host '{other}' (expected markdown|plain)"))
                    }
                })
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag '{flag}'")),
            positional => flags.positional.push(positional.to_string()),
        }
        i += 1;
    }
    Ok(flags)
}

fn run(args: Vec<String>) -> Result<ExitCode, String> {
    let Some(command) = args.first() else {
        println!("{USAGE}");
        return Ok(ExitCode::from(2));
    };
    match command.as_str() {
        "--help" | "help" => {
            println!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        "--version" => {
            println!("lintcite {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        "check" => cmd_check(&args[1..]),
        "parse" => cmd_parse(&args[1..]),
        "explain" => cmd_explain(&args[1..]),
        "fix" => cmd_fix(&args[1..]),
        "editions" => cmd_editions(),
        other => Err(format!("unknown subcommand '{other}' (see --help)")),
    }
}

fn session(edition: &str) -> Result<Session, String> {
    Session::new(edition).map_err(|e| e.to_string())
}

/// Read one input: a file path, or stdin for `-`.
fn read_input(path: &str) -> Result<(String, String, HostKind), String> {
    if path == "-" {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .map_err(|e| format!("stdin: {e}"))?;
        Ok(("<stdin>".to_string(), text, HostKind::Markdown))
    } else {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        Ok((path.to_string(), text, lintcite::host_for_path(path)))
    }
}

fn cmd_check(rest: &[String]) -> Result<ExitCode, String> {
    let flags = parse_flags(rest)?;
    let s = session(&flags.edition)?;
    let inputs = if flags.positional.is_empty() {
        vec!["-".to_string()]
    } else {
        flags.positional.clone()
    };

    let mut per_file: Vec<(String, Vec<Diagnostic>)> = Vec::new();
    let mut sources: Vec<String> = Vec::new();
    for path in &inputs {
        let (label, text, detected_host) = read_input(path)?;
        let host = flags.host.unwrap_or(detected_host);
        per_file.push((label, s.lint(&text, host)));
        sources.push(text);
    }

    let total: usize = per_file.iter().map(|(_, d)| d.len()).sum();
    match flags.format.as_str() {
        "json" => println!("{}", output::json_document(s.edition_id(), &per_file)),
        _ => {
            for ((path, diags), source) in per_file.iter().zip(&sources) {
                for d in diags {
                    println!("{}", output::text_line(path, source, d));
                }
            }
            eprintln!(
                "lintcite: {} diagnostic(s) across {} input(s)",
                per_file.iter().map(|(_, d)| d.len()).sum::<usize>(),
                per_file.len()
            );
        }
    }
    if total == 0 {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

fn cmd_parse(rest: &[String]) -> Result<ExitCode, String> {
    let flags = parse_flags(rest)?;
    if flags.positional.is_empty() {
        return Err("parse needs a citation string".to_string());
    }
    let citation_text = flags.positional.join(" ");
    let s = session(&flags.edition)?;
    let citation = s.parse_citation(&citation_text);
    println!("{citation:#?}");
    Ok(ExitCode::SUCCESS)
}

fn cmd_explain(rest: &[String]) -> Result<ExitCode, String> {
    let s = session("aglc4")?;
    match rest.first().map(String::as_str) {
        Some("--all") => {
            for e in s.explain_all() {
                println!(
                    "{}  [{} | AGLC4 r {}]  {}",
                    e.code,
                    output::severity_label(e.severity),
                    e.aglc_ref.rule,
                    e.summary
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        Some(code) => match s.explain(code) {
            Some(e) => {
                println!("{}: {}", e.code, e.summary);
                println!("severity:   {}", output::severity_label(e.severity));
                println!("AGLC4 rule: {} ({})", e.aglc_ref.rule, e.aglc_ref.anchor);
                match &e.fix {
                    Some(fix) => println!("fix-it:     {fix}"),
                    None => println!("fix-it:     none (no safe transform)"),
                }
                println!("provenance: {}", e.provenance);
                Ok(ExitCode::SUCCESS)
            }
            None => Err(format!("unknown diagnostic code '{code}'")),
        },
        None => Err("explain needs a diagnostic code or --all".to_string()),
    }
}

fn cmd_fix(rest: &[String]) -> Result<ExitCode, String> {
    let flags = parse_flags(rest)?;
    let Some(path) = flags.positional.first() else {
        return Err("fix needs a file path".to_string());
    };
    let (_, text, detected_host) = read_input(path)?;
    let host = flags.host.unwrap_or(detected_host);
    let s = session(&flags.edition)?;
    let result = s.fix(&text, host);
    print!("{}", result.fixed);
    eprintln!("lintcite: applied {} fix(es)", result.applied);
    Ok(ExitCode::SUCCESS)
}

fn cmd_editions() -> Result<ExitCode, String> {
    let s = session("aglc4")?;
    for e in s.editions().map_err(|e| e.to_string())? {
        println!("{}  {}  ({})", e.id, e.label, e.citation);
    }
    Ok(ExitCode::SUCCESS)
}
