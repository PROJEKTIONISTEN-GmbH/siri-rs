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

/// The messages that carry an `AccountKey` — a secret, which a request logged with
/// `{:?}` would otherwise hand to whoever reads the log.
const KEYED: &[&str] = &[
    "CapabilitiesRequest",
    "CheckStatusRequest",
    "DataReadyNotification",
    "DataSupplyRequest",
    "HeartbeatNotification",
    "LinesRequest",
    "ProductCategoriesRequest",
    "ServiceFeaturesRequest",
    "ServiceRequest",
    "StopPointsRequest",
    "SubscriptionRequest",
    "TerminateSubscriptionRequest",
    "VehicleFeaturesRequest",
];

/// Every struct with an `account_key` field is one the test below shows without it.
#[test]
fn the_messages_carrying_an_account_key_are_these() {
    let mut keyed = Vec::new();
    for path in source_files() {
        let source = std::fs::read_to_string(&path).expect("readable source");
        let mut current = None;
        for line in source.lines() {
            if let Some(rest) = line.strip_prefix("pub struct ") {
                current = rest.split(|c: char| !c.is_alphanumeric()).next().map(str::to_owned);
            }
            if line.trim_start().starts_with("pub account_key:") {
                keyed.push(current.clone().unwrap_or_else(|| panic!("{}: a field outside a struct", display(&path))));
            }
        }
    }
    keyed.sort();
    assert_eq!(keyed, KEYED);
}

/// A message's `Debug` output says that it carries a key, never which.
#[test]
fn an_account_key_is_not_shown_in_debug_output() {
    use chrono::DateTime;
    use siri_rs::framework::{
        CapabilitiesRequest, CheckStatusRequest, DataReadyNotification, DataSupplyRequest,
        HeartbeatNotification, LinesRequest, ProductCategoriesRequest, ServiceFeaturesRequest,
        ServiceRequest, StopPointsRequest, SubscriptionRequest, TerminateSubscriptionRequest,
        VehicleFeaturesRequest,
    };
    use siri_rs::Siri;

    const KEY: &str = "s3cr3t-4cc0unt-k3y";
    let now = DateTime::parse_from_rfc3339("2004-12-17T09:30:47-05:00").expect("valid instant");
    macro_rules! keyed {
        ($message:expr) => {{
            let mut message = $message;
            message.account_id = Some("EREWHON-ACCOUNT".into());
            message.account_key = Some(KEY.into());
            (stringify!($message), format!("{message:?}"), format!("{:?}", Siri::new("2.0", message)))
        }};
    }
    let shown = [
        keyed!(CapabilitiesRequest::new(now, "EREWHON", Vec::new())),
        keyed!(CheckStatusRequest::new(now, "EREWHON")),
        keyed!(DataReadyNotification::new(now, "KUBRICK")),
        keyed!(DataSupplyRequest::new(now, "EREWHON")),
        keyed!(HeartbeatNotification::healthy(now, "KUBRICK", now)),
        keyed!(LinesRequest::new(now, "EREWHON")),
        keyed!(ProductCategoriesRequest::new(now, "EREWHON")),
        keyed!(ServiceFeaturesRequest::new(now, "EREWHON")),
        keyed!(ServiceRequest::new(now, "EREWHON", Vec::new())),
        keyed!(StopPointsRequest::new(now, "EREWHON")),
        keyed!(SubscriptionRequest::new(now, "EREWHON", Vec::new())),
        keyed!(TerminateSubscriptionRequest::all(now, "EREWHON")),
        keyed!(VehicleFeaturesRequest::new(now, "EREWHON")),
    ];
    assert_eq!(shown.len(), KEYED.len());
    for (message, alone, enveloped) in shown {
        for debug in [alone, enveloped] {
            assert!(!debug.contains(KEY), "{message} shows its key: {debug}");
            assert!(debug.contains("account_key: Some(<redacted>)"), "{message} says it carries one: {debug}");
            assert!(debug.contains("EREWHON-ACCOUNT"), "{message} still shows the rest: {debug}");
        }
    }
}

/// Every field that reads an `xsd:dateTime` reads it through the crate's own reader,
/// which says why a timestamp without an offset, or at hour 24, is not read; a field
/// that read it plainly would fail such a document in chrono's words alone.
#[test]
fn every_timestamp_field_says_why_a_timestamp_does_not_read() {
    let mut fields = 0;
    let mut plain = Vec::new();
    for path in source_files() {
        let source = std::fs::read_to_string(&path).expect("readable source");
        let lines: Vec<&str> = source.lines().collect();
        for (number, line) in lines.iter().enumerate() {
            let declaration = line.trim();
            // A macro that declares its fields as `name: Type => "Element"` wraps
            // them in `Option`, so they read as optional.
            let reader = if declaration.contains(": DateTime<FixedOffset> =>")
                || declaration.contains(": Option<DateTime<FixedOffset>>,")
            {
                "optional"
            } else if declaration.contains(": DateTime<FixedOffset>,") {
                "required"
            } else {
                continue;
            };
            let attributes = attributes_above(&lines, number);
            if !attributes.iter().any(|attribute| attribute.contains("serde")) {
                continue;
            }
            fields += 1;
            let wanted = format!("deserialize_with = \"crate::xml::date_time::{reader}\"");
            if !attributes.iter().any(|attribute| attribute.contains(&wanted)) {
                plain.push(format!("{}:{}: {declaration}", display(&path), number + 1));
            }
        }
    }
    assert!(fields > 250, "the scan found only {fields} timestamp fields");
    assert!(plain.is_empty(), "{} timestamp fields read plainly:\n{}", plain.len(), plain.join("\n"));
}

/// The attribute lines above the field declared on line `field`, doc comments
/// skipped, an attribute written over several lines included whole.
fn attributes_above<'a>(lines: &[&'a str], field: usize) -> Vec<&'a str> {
    let mut found = Vec::new();
    let mut inside = false;
    for line in lines[..field].iter().rev().map(|line| line.trim()) {
        if inside {
            found.push(line);
            inside = !line.starts_with("#[");
        } else if line.starts_with("//") {
            continue;
        } else if line.starts_with("#[") && line.ends_with(']') {
            found.push(line);
        } else if line == ")]" {
            found.push(line);
            inside = true;
        } else {
            break;
        }
    }
    found
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
