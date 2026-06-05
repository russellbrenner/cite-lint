//! Architecture tests (plan 07 T9): the invariants are CI-checked facts,
//! not aspirations. These read the workspace's own manifests and sources,
//! so a violating change fails here before review sees it.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root resolves")
}

/// Parse the `lintcite-*` path dependencies out of a crate manifest.
/// (Line-based on purpose: manifests are ours and simple.)
fn lintcite_deps(manifest: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_deps = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = line == "[dependencies]";
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = line.split_once('=') {
            let name = name.trim();
            if name.starts_with("lintcite") {
                deps.push(name.to_string());
            }
        }
    }
    deps.sort();
    deps
}

/// Invariant 1: the dependency graph is one-way —
/// data → core → host → lintcite(SDK) → {cli, lsp}.
#[test]
fn dependency_graph_is_one_way() {
    let expected: BTreeMap<&str, Vec<&str>> = BTreeMap::from([
        ("lintcite-data", vec![]),
        ("lintcite-core", vec!["lintcite-data"]),
        ("lintcite-host", vec!["lintcite-core"]),
        (
            "lintcite",
            vec!["lintcite-core", "lintcite-data", "lintcite-host"],
        ),
        ("lintcite-cli", vec!["lintcite"]),
        ("lintcite-lsp", vec!["lintcite"]),
    ]);
    let root = workspace_root();
    for (krate, allowed) in &expected {
        let manifest = fs::read_to_string(root.join("crates").join(krate).join("Cargo.toml"))
            .unwrap_or_else(|e| panic!("read {krate}/Cargo.toml: {e}"));
        let actual = lintcite_deps(&manifest);
        assert_eq!(
            &actual.iter().map(String::as_str).collect::<Vec<_>>(),
            allowed,
            "crate '{krate}' violates the one-way dependency graph \
             (invariant 1): {actual:?}"
        );
    }
}

/// Invariant 2 (plan 01 T7): no controlled vocabulary term may be
/// string-literalled in `lintcite-core` — rules read vocab from data.
/// Test modules (which legitimately use vocab as fixtures) sit at the end
/// of each file behind `#[cfg(test)]`; everything before it is scanned.
#[test]
fn core_never_inlines_controlled_vocabulary() {
    let tables = lintcite_data::load("aglc4").expect("edition loads");
    let mut vocab: Vec<String> = Vec::new();
    for r in tables.reporters() {
        vocab.push(format!("\"{}\"", r.abbrev));
    }
    for c in tables.courts() {
        vocab.push(format!("\"{}\"", c.id));
    }

    let src_dir = workspace_root().join("crates/lintcite-core/src");
    let mut scanned = 0usize;
    for entry in fs::read_dir(&src_dir).expect("core src dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let full = fs::read_to_string(&path).expect("read source");
        // Scan only non-test code (convention: test modules are terminal).
        let non_test = match full.find("#[cfg(test)]") {
            Some(idx) => &full[..idx],
            None => &full,
        };
        scanned += 1;
        for term in &vocab {
            assert!(
                !non_test.contains(term.as_str()),
                "{} inlines controlled vocab {} — read it from \
                 lintcite-data instead (invariant 2)",
                path.display(),
                term
            );
        }
    }
    assert!(scanned >= 5, "expected to scan core sources, got {scanned}");
}

/// P3: nothing vector-store- or network-shaped may reach the engine crates.
#[test]
fn lint_path_has_no_io_dependencies() {
    let root = workspace_root();
    for krate in ["lintcite-data", "lintcite-core", "lintcite-host"] {
        let manifest = fs::read_to_string(root.join("crates").join(krate).join("Cargo.toml"))
            .expect("manifest");
        let mut in_deps = false;
        for line in manifest.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_deps = line == "[dependencies]";
                continue;
            }
            if in_deps && !line.is_empty() && !line.starts_with('#') {
                let name = line.split('=').next().map(str::trim).unwrap_or("");
                assert!(
                    name.starts_with("lintcite"),
                    "engine crate '{krate}' grew a third-party dependency \
                     '{name}' — that requires the ADR + cargo-deny gate \
                     (docs/adr/0001-m1-provisional-choices.md)"
                );
            }
        }
    }
}
