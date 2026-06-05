//! A strict, deterministic reader for the TOML subset used by edition data files.
//!
//! What: parses the committed `tables/*.toml` and `rules/*.ir.toml` files.
//! How: line-based recursive structure builder; rejects anything outside the
//! documented subset with a line-numbered error (authoring mistakes surface
//! immediately rather than loading as garbage).
//! Depends on: `std` only.
//!
//! Supported subset (everything else is an error):
//! - `# comments`, blank lines
//! - `[table.header]` and `[[array.of.tables]]` with dotted bare keys
//! - `key = "string"` (escapes: `\"` `\\` `\n` `\t`)
//! - `key = 123` / `key = -7` (i64)
//! - `key = true` / `key = false`
//! - `key = ["a", "b"]` (single-line array of strings)
//!
//! This is a provisional stand-in for the `toml` crate, adopted when the
//! dependency policy opens up — see docs/adr/0001-m1-provisional-choices.md.
//! The public data-file schema is unaffected by that swap: these files are
//! valid TOML.

use std::collections::BTreeMap;

use crate::error::DataError;

/// A parsed TOML-subset value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Item {
    /// A quoted string value.
    Str(String),
    /// An integer value.
    Int(i64),
    /// A boolean value.
    Bool(bool),
    /// A single-line array of strings.
    StrArray(Vec<String>),
    /// A nested table (`[header]`).
    Table(Table),
    /// An array of tables (`[[header]]`).
    Tables(Vec<Table>),
}

/// A table: deterministic key order via `BTreeMap`.
pub(crate) type Table = BTreeMap<String, Item>;

/// Parse `src` (the contents of `file`) into a root [`Table`].
pub(crate) fn parse(file: &'static str, src: &str) -> Result<Table, DataError> {
    let mut root = Table::new();
    // Path of the table currently being filled, e.g. ["edition"] or
    // ["rule"] for an array-of-tables entry.
    let mut current: Vec<String> = Vec::new();
    // Whether `current` names an array-of-tables (fill its last element).
    let mut current_is_array = false;

    for (idx, raw) in src.lines().enumerate() {
        let line_no = idx + 1;
        let line = strip_comment(raw);
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("[[") {
            let header = rest.strip_suffix("]]").ok_or_else(|| {
                err_parse(file, line_no, "array-of-tables header must end with ']]'")
            })?;
            let path = parse_dotted(file, line_no, header)?;
            push_array_table(file, line_no, &mut root, &path)?;
            current = path;
            current_is_array = true;
            continue;
        }

        if let Some(rest) = line.strip_prefix('[') {
            let header = rest
                .strip_suffix(']')
                .ok_or_else(|| err_parse(file, line_no, "table header must end with ']'"))?;
            let path = parse_dotted(file, line_no, header)?;
            open_table(file, line_no, &mut root, &path)?;
            current = path;
            current_is_array = false;
            continue;
        }

        // Otherwise: key = value
        let eq = line
            .find('=')
            .ok_or_else(|| err_parse(file, line_no, "expected '[header]' or 'key = value'"))?;
        let key = line[..eq].trim();
        validate_bare_key(file, line_no, key)?;
        let value = parse_value(file, line_no, line[eq + 1..].trim())?;

        let target = resolve_target(file, line_no, &mut root, &current, current_is_array)?;
        if target.contains_key(key) {
            return Err(DataError::Duplicate {
                file,
                id: format!("{} (line {line_no})", key),
            });
        }
        target.insert(key.to_string(), value);
    }

    Ok(root)
}

/// Strip a `#` comment, respecting `#` inside quoted strings.
fn strip_comment(line: &str) -> &str {
    let mut in_str = false;
    let mut escaped = false;
    for (i, ch) in line.char_indices() {
        if in_str {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_str = false;
            }
        } else if ch == '"' {
            in_str = true;
        } else if ch == '#' {
            return &line[..i];
        }
    }
    line
}

fn err_parse(file: &'static str, line: usize, message: &str) -> DataError {
    DataError::Parse {
        file,
        line,
        message: message.to_string(),
    }
}

fn validate_bare_key(file: &'static str, line: usize, key: &str) -> Result<(), DataError> {
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(err_parse(file, line, &format!("invalid bare key '{key}'")));
    }
    Ok(())
}

fn parse_dotted(file: &'static str, line: usize, header: &str) -> Result<Vec<String>, DataError> {
    let parts: Vec<&str> = header.trim().split('.').collect();
    let mut path = Vec::with_capacity(parts.len());
    for part in parts {
        let part = part.trim();
        validate_bare_key(file, line, part)?;
        path.push(part.to_string());
    }
    Ok(path)
}

/// Walk/create nested tables for a `[header]`, erroring on type clashes.
fn open_table(
    file: &'static str,
    line: usize,
    root: &mut Table,
    path: &[String],
) -> Result<(), DataError> {
    let mut node = root;
    for (i, part) in path.iter().enumerate() {
        let last = i == path.len() - 1;
        let entry = node
            .entry(part.clone())
            .or_insert_with(|| Item::Table(Table::new()));
        node = match entry {
            Item::Table(t) => {
                if last && !t.is_empty() {
                    return Err(DataError::Duplicate {
                        file,
                        id: format!("[{}] (line {line})", path.join(".")),
                    });
                }
                t
            }
            Item::Tables(v) => {
                if last {
                    return Err(err_parse(
                        file,
                        line,
                        &format!("'{part}' is an array of tables; use [[{part}]]"),
                    ));
                }
                v.last_mut()
                    .ok_or_else(|| err_parse(file, line, "empty array of tables"))?
            }
            _ => return Err(err_parse(file, line, &format!("'{part}' is not a table"))),
        }
    }
    Ok(())
}

/// Append a new table for a `[[header]]`.
fn push_array_table(
    file: &'static str,
    line: usize,
    root: &mut Table,
    path: &[String],
) -> Result<(), DataError> {
    let (last, parents) = path
        .split_last()
        .ok_or_else(|| err_parse(file, line, "empty array-of-tables header"))?;
    let mut node = root;
    for part in parents {
        let entry = node
            .entry(part.clone())
            .or_insert_with(|| Item::Table(Table::new()));
        node = match entry {
            Item::Table(t) => t,
            Item::Tables(v) => v
                .last_mut()
                .ok_or_else(|| err_parse(file, line, "empty array of tables"))?,
            _ => return Err(err_parse(file, line, &format!("'{part}' is not a table"))),
        };
    }
    match node
        .entry(last.clone())
        .or_insert_with(|| Item::Tables(Vec::new()))
    {
        Item::Tables(v) => {
            v.push(Table::new());
            Ok(())
        }
        _ => Err(err_parse(
            file,
            line,
            &format!("'{last}' already used as a non-array item"),
        )),
    }
}

/// Find the table a `key = value` line should land in.
fn resolve_target<'a>(
    file: &'static str,
    line: usize,
    root: &'a mut Table,
    current: &[String],
    current_is_array: bool,
) -> Result<&'a mut Table, DataError> {
    let mut node = root;
    for (i, part) in current.iter().enumerate() {
        let last = i == current.len() - 1;
        let entry = node
            .get_mut(part)
            .ok_or_else(|| err_parse(file, line, &format!("internal: missing table '{part}'")))?;
        node = match entry {
            Item::Table(t) => t,
            Item::Tables(v) => {
                if last && !current_is_array {
                    return Err(err_parse(
                        file,
                        line,
                        &format!("'{part}' is an array of tables"),
                    ));
                }
                v.last_mut()
                    .ok_or_else(|| err_parse(file, line, "empty array of tables"))?
            }
            _ => return Err(err_parse(file, line, &format!("'{part}' is not a table"))),
        };
    }
    Ok(node)
}

/// Parse a scalar or single-line string-array value.
fn parse_value(file: &'static str, line: usize, raw: &str) -> Result<Item, DataError> {
    if raw.starts_with('"') {
        let (s, rest) = parse_string(file, line, raw)?;
        if !rest.trim().is_empty() {
            return Err(err_parse(file, line, "trailing content after string"));
        }
        return Ok(Item::Str(s));
    }
    if raw == "true" {
        return Ok(Item::Bool(true));
    }
    if raw == "false" {
        return Ok(Item::Bool(false));
    }
    if raw.starts_with('[') {
        let inner = raw
            .strip_prefix('[')
            .and_then(|r| r.strip_suffix(']'))
            .ok_or_else(|| err_parse(file, line, "array must open and close on one line"))?;
        let mut items = Vec::new();
        let mut rest = inner.trim();
        while !rest.is_empty() {
            if !rest.starts_with('"') {
                return Err(err_parse(file, line, "arrays may contain only strings"));
            }
            let (s, after) = parse_string(file, line, rest)?;
            items.push(s);
            rest = after.trim();
            if let Some(r) = rest.strip_prefix(',') {
                rest = r.trim();
            } else if !rest.is_empty() {
                return Err(err_parse(file, line, "expected ',' between items"));
            }
        }
        return Ok(Item::StrArray(items));
    }
    // Integer (optionally negative).
    let body = raw.strip_prefix('-').unwrap_or(raw);
    if !body.is_empty() && body.chars().all(|c| c.is_ascii_digit()) {
        return raw
            .parse::<i64>()
            .map(Item::Int)
            .map_err(|_| err_parse(file, line, "integer out of range"));
    }
    Err(err_parse(
        file,
        line,
        &format!("unsupported value syntax: '{raw}'"),
    ))
}

/// Parse a leading quoted string; return (content, remainder-after-quote).
fn parse_string<'a>(
    file: &'static str,
    line: usize,
    raw: &'a str,
) -> Result<(String, &'a str), DataError> {
    debug_assert!(raw.starts_with('"'));
    let mut out = String::new();
    let mut chars = raw.char_indices().skip(1);
    while let Some((i, ch)) = chars.next() {
        match ch {
            '"' => return Ok((out, &raw[i + 1..])),
            '\\' => match chars.next() {
                Some((_, '"')) => out.push('"'),
                Some((_, '\\')) => out.push('\\'),
                Some((_, 'n')) => out.push('\n'),
                Some((_, 't')) => out.push('\t'),
                _ => return Err(err_parse(file, line, "unsupported escape sequence")),
            },
            other => out.push(other),
        }
    }
    Err(err_parse(file, line, "unterminated string"))
}

// ---------------------------------------------------------------------------
// Typed accessors used by the schema loaders.
// ---------------------------------------------------------------------------

/// Required string field, with a schema-shaped error.
pub(crate) fn req_str<'a>(
    file: &'static str,
    item: &str,
    table: &'a Table,
    key: &str,
) -> Result<&'a str, DataError> {
    match table.get(key) {
        Some(Item::Str(s)) => Ok(s),
        _ => Err(DataError::Validation {
            file,
            item: item.to_string(),
            message: format!("missing required string field '{key}'"),
        }),
    }
}

/// Optional string field.
pub(crate) fn opt_str<'a>(table: &'a Table, key: &str) -> Option<&'a str> {
    match table.get(key) {
        Some(Item::Str(s)) => Some(s),
        _ => None,
    }
}

/// Required sub-table, with a schema-shaped error.
pub(crate) fn req_table<'a>(
    file: &'static str,
    item: &str,
    table: &'a Table,
    key: &str,
) -> Result<&'a Table, DataError> {
    match table.get(key) {
        Some(Item::Table(t)) => Ok(t),
        _ => Err(DataError::Validation {
            file,
            item: item.to_string(),
            message: format!("missing required table '{key}'"),
        }),
    }
}

/// Array-of-tables field (empty if absent).
pub(crate) fn tables<'a>(table: &'a Table, key: &str) -> &'a [Table] {
    match table.get(key) {
        Some(Item::Tables(v)) => v,
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const F: &str = "test.toml";

    #[test]
    fn parses_scalars_and_comments() {
        let t = parse(F, "# header\na = \"x\" # trailing\nb = 7\nc = true\n").expect("parse");
        assert_eq!(t.get("a"), Some(&Item::Str("x".to_string())));
        assert_eq!(t.get("b"), Some(&Item::Int(7)));
        assert_eq!(t.get("c"), Some(&Item::Bool(true)));
    }

    #[test]
    fn hash_inside_string_is_not_a_comment() {
        let t = parse(F, "a = \"x # y\"\n").expect("parse");
        assert_eq!(t.get("a"), Some(&Item::Str("x # y".to_string())));
    }

    #[test]
    fn parses_string_escapes() {
        let t = parse(F, r#"a = "q\"u\\o\nt""#).expect("parse");
        assert_eq!(t.get("a"), Some(&Item::Str("q\"u\\o\nt".to_string())));
    }

    #[test]
    fn parses_nested_tables_and_arrays_of_tables() {
        let src = "\n[edition]\nid = \"aglc4\"\n[edition.source]\nurl = \"u\"\n\n[[reporter]]\nabbrev = \"CLR\"\n[[reporter]]\nabbrev = \"VR\"\n";
        let t = parse(F, src).expect("parse");
        let edition = match t.get("edition") {
            Some(Item::Table(t)) => t,
            other => panic!("edition not a table: {other:?}"),
        };
        assert_eq!(opt_str(edition, "id"), Some("aglc4"));
        let source = req_table(F, "edition", edition, "source").expect("source");
        assert_eq!(opt_str(source, "url"), Some("u"));
        let reps = tables(&t, "reporter");
        assert_eq!(reps.len(), 2);
        assert_eq!(opt_str(&reps[0], "abbrev"), Some("CLR"));
        assert_eq!(opt_str(&reps[1], "abbrev"), Some("VR"));
    }

    #[test]
    fn parses_string_arrays() {
        let t = parse(F, "a = [\"x\", \"y\"]\nempty = []\n").expect("parse");
        assert_eq!(
            t.get("a"),
            Some(&Item::StrArray(vec!["x".to_string(), "y".to_string()]))
        );
        assert_eq!(t.get("empty"), Some(&Item::StrArray(vec![])));
    }

    #[test]
    fn rejects_duplicate_keys() {
        let err = parse(F, "a = 1\na = 2\n").expect_err("duplicate");
        assert!(matches!(err, DataError::Duplicate { .. }), "{err}");
    }

    #[test]
    fn rejects_unsupported_syntax_with_line_number() {
        let err = parse(F, "a = 1\nb = 1.5\n").expect_err("float unsupported");
        match err {
            DataError::Parse { line, .. } => assert_eq!(line, 2),
            other => panic!("wrong error: {other}"),
        }
    }

    #[test]
    fn rejects_unterminated_string() {
        assert!(parse(F, "a = \"oops\n").is_err());
    }

    #[test]
    fn keys_after_array_header_land_in_last_element() {
        let src = "[[r]]\na = \"1\"\n[[r]]\na = \"2\"\nb = \"x\"\n";
        let t = parse(F, src).expect("parse");
        let rs = tables(&t, "r");
        assert_eq!(rs.len(), 2);
        assert_eq!(opt_str(&rs[1], "b"), Some("x"));
        assert_eq!(opt_str(&rs[0], "b"), None);
    }

    #[test]
    fn negative_integers_parse() {
        let t = parse(F, "a = -42\n").expect("parse");
        assert_eq!(t.get("a"), Some(&Item::Int(-42)));
    }
}
