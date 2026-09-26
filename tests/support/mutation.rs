//! Rewriting an example document so that it carries a fault.
//!
//! The tests of lenient reading break the official examples one element at a
//! time — empty a timestamp, take a mandatory element out, put a word where a
//! number goes — and expect the reader to leave out exactly what the fault
//! touches. Elements are addressed by name and by position among the elements
//! of that name in document order, which is how a reader of the fixture finds
//! them too; the text between the tags is left as the example wrote it.

use std::ops::Range;

/// The byte range of the `nth` element named `name` (counted from 0 over the
/// start tags in document order), from the `<` of its start tag to the `>` of
/// its end tag.
pub fn element(xml: &str, name: &str, nth: usize) -> Range<usize> {
    let start = start_tags(xml, name)
        .nth(nth)
        .unwrap_or_else(|| panic!("the document has no <{name}> number {nth}"));
    let mut depth = 0usize;
    let mut at = start;
    loop {
        let open = xml[at..].find('<').map_or_else(|| panic!("<{name}> is never closed"), |i| at + i);
        if xml[open..].starts_with("<!--") {
            at = xml[open..].find("-->").map_or_else(|| panic!("a comment is never closed"), |i| open + i + 3);
            continue;
        }
        let close = xml[open..].find('>').map_or_else(|| panic!("a tag is never closed"), |i| open + i + 1);
        let tag = &xml[open..close];
        if opens(tag, name) {
            if tag.ends_with("/>") {
                if depth == 0 {
                    return start..close;
                }
            } else {
                depth += 1;
            }
        } else if tag == format!("</{name}>") {
            depth -= 1;
            if depth == 0 {
                return start..close;
            }
        }
        at = close;
    }
}

/// The text of the `nth` element named `name`, tags included.
pub fn copy(xml: &str, name: &str, nth: usize) -> String {
    xml[element(xml, name, nth)].to_string()
}

/// How many elements named `name` the document holds.
pub fn count(xml: &str, name: &str) -> usize {
    start_tags(xml, name).count()
}

/// The document without the `nth` element named `name`.
pub fn remove(xml: &str, name: &str, nth: usize) -> String {
    replace(xml, name, nth, "")
}

/// The document with the `nth` element named `name` replaced by `with`.
pub fn replace(xml: &str, name: &str, nth: usize, with: &str) -> String {
    let range = element(xml, name, nth);
    let mut out = xml.to_string();
    out.replace_range(range, with);
    out
}

/// The document with the `nth` element named `name` written empty, `<Name/>`.
pub fn emptied(xml: &str, name: &str, nth: usize) -> String {
    replace(xml, name, nth, &format!("<{name}/>"))
}

/// The document with the `nth` element named `name` holding `text` and nothing else.
pub fn with_text(xml: &str, name: &str, nth: usize, text: &str) -> String {
    replace(xml, name, nth, &format!("<{name}>{text}</{name}>"))
}

/// The document with `what` inserted right after the `nth` element named `name`.
pub fn insert_after(xml: &str, name: &str, nth: usize, what: &str) -> String {
    let range = element(xml, name, nth);
    let mut out = xml.to_string();
    out.insert_str(range.end, what);
    out
}

/// The document with `what` inserted right before the `nth` element named `name`.
pub fn insert_before(xml: &str, name: &str, nth: usize, what: &str) -> String {
    let range = element(xml, name, nth);
    let mut out = xml.to_string();
    out.insert_str(range.start, what);
    out
}

/// The offsets of every start tag (or empty-element tag) named `name`, in
/// document order, comments skipped.
fn start_tags<'a>(xml: &'a str, name: &'a str) -> impl Iterator<Item = usize> + 'a {
    let mut at = 0;
    std::iter::from_fn(move || loop {
        let open = at + xml[at..].find('<')?;
        if xml[open..].starts_with("<!--") {
            at = open + xml[open..].find("-->")? + 3;
            continue;
        }
        let close = open + xml[open..].find('>')? + 1;
        at = close;
        if opens(&xml[open..close], name) {
            return Some(open);
        }
    })
}

/// Whether `tag` is a start tag or an empty-element tag of `name`.
fn opens(tag: &str, name: &str) -> bool {
    tag.strip_prefix('<')
        .and_then(|rest| rest.strip_prefix(name))
        .and_then(|rest| rest.chars().next())
        .is_some_and(|next| next == '>' || next == '/' || next.is_whitespace())
}
