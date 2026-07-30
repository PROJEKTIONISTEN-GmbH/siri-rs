//! Checks that nothing internal to how this crate was written leaks into what is
//! published.
//!
//! A library that ships with references to its authors' issue tracker, working
//! directories or half-finished intentions reads as unfinished. This is cheap to
//! check mechanically, so it is checked mechanically.

use std::path::{Path, PathBuf};

/// Substrings that must not appear in anything shipped.
const FORBIDDEN: &[&str] = &[
    "TODO",
    "FIXME",
    "XXX",
    "HACK",
    "/workspace",
    "/single_repos",
    "monorepo",
];

#[test]
fn nothing_internal_is_shipped() {
    let mut findings = Vec::new();

    for path in published_files() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            for needle in FORBIDDEN {
                if line.contains(needle) {
                    findings.push(format!(
                        "{}:{}: {needle} — {}",
                        display(&path),
                        number + 1,
                        line.trim()
                    ));
                }
            }
            if let Some(reference) = development_reference(line) {
                findings.push(format!(
                    "{}:{}: {reference} — {}",
                    display(&path),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        findings.is_empty(),
        "the crate refers to how it was developed:\n{}",
        findings.join("\n")
    );
}

/// Finds a reference to a numbered unit of development work, e.g. `sprint 42`.
fn development_reference(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let mut haystack = lower.as_str();
    while let Some(at) = haystack.find("sprint") {
        let rest = haystack[at + "sprint".len()..].trim_start_matches([' ', '-', '_']);
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            return Some(format!("sprint {}", rest.split_whitespace().next()?));
        }
        haystack = &haystack[at + "sprint".len()..];
    }
    None
}

/// Every file that goes into the published package, except the fixtures, which are
/// the standards body's own material and are covered by their own notice.
fn published_files() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = vec![
        root.join("Cargo.toml"),
        root.join("README.md"),
        root.join("LICENSE-MIT"),
        root.join("LICENSE-APACHE"),
    ];
    for directory in ["src", "examples"] {
        collect(&root.join(directory), &mut out);
    }
    for entry in std::fs::read_dir(root.join("tests")).expect("readable test directory") {
        let path = entry.expect("readable directory entry").path();
        if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    collect(&root.join("tests/support"), &mut out);
    out.sort();
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
    {
        let path = entry.expect("readable directory entry").path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs" || e == "md") {
            out.push(path);
        }
    }
}

fn display(path: &Path) -> String {
    path.strip_prefix(env!("CARGO_MANIFEST_DIR"))
        .unwrap_or(path)
        .display()
        .to_string()
}

#[test]
fn the_check_would_catch_a_leak() {
    assert!(development_reference("added in sprint 42").is_some());
    assert!(development_reference("Sprint-500 follow-up").is_some());
    assert!(development_reference("a sprint finish").is_none());
    assert!(development_reference("the situation is published").is_none());
}

/// Every file the manifest promises is where it says it is.
#[test]
fn the_manifest_points_at_files_that_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in ["README.md", "LICENSE-MIT", "LICENSE-APACHE"] {
        assert!(root.join(file).is_file(), "{file} is missing");
    }
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("readable manifest");
    assert!(manifest.contains("readme = \"README.md\""));
    assert!(manifest.contains("license = \"MIT OR Apache-2.0\""));
}
