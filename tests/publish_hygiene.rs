//! Checks that nothing internal to how this crate was written leaks into what is
//! published.
//!
//! A library that ships with references to its authors' issue tracker, working
//! directories or half-finished intentions reads as unfinished, and so does one whose
//! front page sends the reader to an example that is not there. Both are cheap to
//! check mechanically, so they are checked mechanically.

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
    for directory in ["src", "examples", "benches"] {
        collect(&root.join(directory), &mut out);
    }
    for entry in std::fs::read_dir(root.join("tests")).expect("readable test directory") {
        let path = entry.expect("readable directory entry").path();
        if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    collect(&root.join("tests/support"), &mut out);
    // This file spells out the very substrings it searches for, so scanning it would
    // always report itself. That it still recognises a leak is asserted separately by
    // `the_check_would_catch_a_leak`.
    out.retain(|path| path.file_name().map_or(true, |name| name != THIS_FILE));
    out.sort();
    out
}

/// The checker's own file name, excluded from the scan above.
const THIS_FILE: &str = "publish_hygiene.rs";

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

/// The front page and the examples say the same thing: every example is offered to
/// the reader, and every example the reader is told to run is there.
#[test]
fn the_readme_and_the_examples_agree() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(root.join("README.md")).expect("readable front page");

    let mut shipped: Vec<String> = std::fs::read_dir(root.join("examples"))
        .expect("readable example directory")
        .map(|entry| entry.expect("readable directory entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "rs"))
        .map(|path| {
            path.file_stem()
                .expect("an example has a name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    shipped.sort();

    assert_eq!(examples_offered_in(&readme), shipped);
}

/// The examples a text tells the reader to run, in a stable order.
fn examples_offered_in(text: &str) -> Vec<String> {
    let mut offered: Vec<String> = text
        .split("--example ")
        .skip(1)
        .filter_map(|rest| rest.split_whitespace().next())
        .map(|name| {
            name.trim_end_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .to_owned()
        })
        .collect();
    offered.sort();
    offered.dedup();
    offered
}

#[test]
fn the_check_would_catch_an_example_the_front_page_forgot() {
    assert_eq!(
        examples_offered_in("Run it with `cargo run --example sx_endpoint`."),
        ["sx_endpoint"]
    );
    assert_eq!(
        examples_offered_in("cargo run --example one  # a comment\ncargo run --example two\n"),
        ["one", "two"]
    );
    assert!(examples_offered_in("no examples are offered here").is_empty());
}

/// The front page is read as plain Markdown — on the repository page and on
/// crates.io — where rustdoc's doc-test scaffolding does not survive: a hidden `#`
/// line is shown verbatim, and a `no_run` or `ignore` fence attribute is noise in
/// front of the reader. So the front page carries none of it, and the compile check
/// lives where the scaffolding is invisible instead: the examples and the rustdoc.
#[test]
fn the_front_page_carries_no_doc_test_scaffolding() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(root.join("README.md")).expect("readable front page");

    let findings = doc_test_scaffolding_in(&readme);
    assert!(
        findings.is_empty(),
        "the front page shows doc-test scaffolding to its readers:\n{}",
        findings.join("\n")
    );
}

/// The languages a fenced block on the front page may be written in. Anything else
/// is a rustdoc attribute rather than a language.
const FENCE_LANGUAGES: &[&str] = &["rust", "sh", "text"];

/// Reports every fenced block whose fence carries a rustdoc attribute and every
/// line inside one that rustdoc would hide.
fn doc_test_scaffolding_in(text: &str) -> Vec<String> {
    let mut findings = Vec::new();
    let mut open = false;
    for (number, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(info) = trimmed.strip_prefix("```") {
            if open {
                open = false;
            } else {
                open = true;
                if !FENCE_LANGUAGES.contains(&info) {
                    findings.push(format!("{}: fence ```{info}", number + 1));
                }
            }
        } else if open && trimmed.starts_with('#') {
            findings.push(format!("{}: hidden line — {trimmed}", number + 1));
        }
    }
    findings
}

#[test]
fn the_check_would_catch_doc_test_scaffolding() {
    assert!(doc_test_scaffolding_in("```rust\nlet answer = 42;\n```\n").is_empty());
    assert!(doc_test_scaffolding_in("# A heading\n\nAnd prose about `#`.\n").is_empty());
    assert!(
        doc_test_scaffolding_in("```sh\ncargo run --example one  # in one terminal\n```\n")
            .is_empty(),
        "a comment at the end of a line is not scaffolding"
    );
    assert_eq!(
        doc_test_scaffolding_in("```rust\n# fn run() {\nlet answer = 42;\n# }\n```\n").len(),
        2
    );
    assert_eq!(
        doc_test_scaffolding_in("```rust,no_run\nlet answer = 42;\n```\n"),
        ["1: fence ```rust,no_run"]
    );
    assert_eq!(
        doc_test_scaffolding_in("```\nlet answer = 42;\n```\n"),
        ["1: fence ```"],
        "a block with no language is not a language this page uses"
    );
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
