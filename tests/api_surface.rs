//! The shape of the public surface, held in place so that the next schema revision
//! is the release it is meant to be.
//!
//! The rule is stated on the front page: every public enum is non-exhaustive, so
//! that a variant a later schema adds is not a breaking change; the state the hub
//! keeps for itself is not a public field; and `Service` is sealed, which the
//! trait's own documentation pins with a test that must fail to compile. What can
//! be read off the source is checked here, the way the enumeration count is.

use std::path::{Path, PathBuf};

/// Every public enum in the crate carries `#[non_exhaustive]`.
#[test]
fn every_public_enum_is_non_exhaustive() {
    let mut exhaustive = Vec::new();
    for path in source_files() {
        let source = std::fs::read_to_string(&path).expect("readable source");
        let lines: Vec<&str> = source.lines().collect();
        for (number, line) in lines.iter().enumerate() {
            let Some(name) = declared_enum(line) else {
                continue;
            };
            let attributes = lines[..number]
                .iter()
                .rev()
                .take_while(|preceding| {
                    let preceding = preceding.trim_start();
                    preceding.starts_with("#[") || preceding.starts_with("///")
                })
                .any(|attribute| attribute.contains("non_exhaustive"));
            if !attributes {
                exhaustive.push(format!("{}:{}: {name}", display(&path), number + 1));
            }
        }
    }
    assert!(
        exhaustive.is_empty(),
        "public enums a schema revision could grow, declared exhaustive:\n{}",
        exhaustive.join("\n")
    );
}

/// The state the hub keeps for a subscription, and the configuration a producer
/// runs with, are read through methods rather than fields.
#[test]
fn the_hubs_own_state_is_not_a_public_field() {
    let source = std::fs::read_to_string(root().join("src/pubsub/producer.rs"))
        .expect("readable producer");
    for name in ["ProducerConfig", "Subscription<S: Service>"] {
        let declaration = format!("pub struct {name} {{");
        let start = source
            .find(&declaration)
            .unwrap_or_else(|| panic!("{name} is declared in the producer"));
        let body = &source[start + declaration.len()..];
        let body = &body[..body.find("\n}").expect("the struct ends")];
        assert!(
            !body.contains("pub "),
            "{name} exposes a field:\n{body}"
        );
    }
}

/// The name of a public enum declared on `line`, if one is — `$name` inside a
/// macro counts, since the macro declares every enum it expands to.
fn declared_enum(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("pub enum ")?;
    let name = rest
        .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
        .next()?;
    (!name.is_empty()).then_some(name)
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn source_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect(&root().join("src"), &mut out);
    out.sort();
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
    {
        let path = entry.expect("readable directory entry").path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn display(path: &Path) -> String {
    path.strip_prefix(root())
        .unwrap_or(path)
        .display()
        .to_string()
}
