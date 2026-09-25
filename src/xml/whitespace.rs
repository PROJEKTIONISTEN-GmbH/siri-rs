//! Keeping character data that is nothing but whitespace.
//!
//! An element whose content is only whitespace — `<Summary>   </Summary>` — holds
//! that text: the schema accepts it where it asks for at least one character, and a
//! reader that keeps documents whole has to keep it. The serde layer of the XML
//! reader drops such text before any type sees it, and cannot be told not to. So
//! the document is looked over first, and the text of every such element is wrapped
//! in a CDATA section, which that layer passes through untouched: the same
//! character data, spelled so that it survives.
//!
//! Only a leaf's content is kept. The whitespace that indents one element inside
//! another borders a tag on either side, and is dropped as it always was.
//!
//! Most documents hold no such element and are returned borrowed after a scan that
//! parses nothing: a candidate is an end tag that follows whitespace that follows a
//! start tag. Only a document with a candidate is tokenised, which settles what a
//! scan cannot — a start tag inside a comment or a CDATA section is no element —
//! and it is rewritten once.

use std::borrow::Cow;
use std::ops::Range;

use quick_xml::events::Event;
use quick_xml::Reader;

/// Rewrites every element whose content is whitespace alone so that the content
/// survives the serde layer's trimming; returns the input unchanged when it has no
/// such element.
pub(crate) fn preserve(xml: &str) -> Cow<'_, str> {
    if !may_hold_blank_content(xml) {
        return Cow::Borrowed(xml);
    }
    let blanks = blank_content(xml);
    if blanks.is_empty() {
        return Cow::Borrowed(xml);
    }

    let mut out = String::with_capacity(xml.len() + blanks.len() * CDATA_OVERHEAD);
    let mut copied = 0;
    for blank in blanks {
        out.push_str(&xml[copied..blank.start]);
        out.push_str("<![CDATA[");
        out.push_str(&xml[blank.clone()]);
        out.push_str("]]>");
        copied = blank.end;
    }
    out.push_str(&xml[copied..]);
    Cow::Owned(out)
}

/// What wrapping one text in a CDATA section adds to the document.
const CDATA_OVERHEAD: usize = "<![CDATA[]]>".len();

/// Whether the document may hold an element whose content is whitespace alone:
/// an end tag preceded by whitespace preceded by what looks like a start tag.
///
/// The scan is over bytes and can be misled by a tag spelled inside a comment or a
/// CDATA section, so a `true` is a reason to tokenise, not a finding. It cannot
/// miss an element: the only way to be sure nothing is there is to have looked at
/// every end tag.
fn may_hold_blank_content(xml: &str) -> bool {
    let bytes = xml.as_bytes();
    let mut from = 0;
    while let Some(at) = xml[from..].find("</") {
        let end_tag = from + at;
        from = end_tag + 2;
        let mut text_start = end_tag;
        while text_start > 0 && bytes[text_start - 1].is_ascii_whitespace() {
            text_start -= 1;
        }
        if text_start == end_tag || text_start == 0 || bytes[text_start - 1] != b'>' {
            continue;
        }
        if closes_a_start_tag(bytes, text_start - 1) {
            return true;
        }
    }
    false
}

/// Whether the `>` at `close` ends a start tag rather than an end tag, an empty
/// element, a comment, a processing instruction or a CDATA section, and is not a
/// `>` inside an attribute value.
fn closes_a_start_tag(bytes: &[u8], close: usize) -> bool {
    let Some(open) = bytes[..close].iter().rposition(|&byte| byte == b'<') else {
        return false;
    };
    let tag = &bytes[open + 1..close];
    match tag.first() {
        None | Some(b'/' | b'!' | b'?') => return false,
        Some(_) => {}
    }
    if tag.last() == Some(&b'/') {
        return false;
    }
    let mut quote = None;
    for &byte in tag {
        match quote {
            Some(open) if byte == open => quote = None,
            Some(_) => {}
            None if byte == b'"' || byte == b'\'' => quote = Some(byte),
            None => {}
        }
    }
    quote.is_none()
}

/// The byte ranges of every text that is whitespace alone and is the whole content
/// of its element, in document order.
///
/// A document the tokeniser cannot read yields nothing: it is handed on as it is,
/// and the reader that follows reports the fault with its own position.
fn blank_content(xml: &str) -> Vec<Range<usize>> {
    let mut reader = Reader::from_str(xml);
    let mut blanks = Vec::new();
    let mut after_start = false;
    let mut pending: Option<Range<usize>> = None;

    loop {
        let event = match reader.read_event() {
            Ok(event) => event,
            Err(_) => return Vec::new(),
        };
        match event {
            Event::Eof => break,
            Event::Start(_) => {
                after_start = true;
                pending = None;
            }
            Event::Text(text) if after_start && text.iter().all(u8::is_ascii_whitespace) => {
                let end = usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX);
                pending = Some(end - text.len()..end);
                after_start = false;
            }
            Event::End(_) => {
                blanks.extend(pending.take());
                after_start = false;
            }
            _ => {
                after_start = false;
                pending = None;
            }
        }
    }
    blanks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_document_without_blank_content_is_returned_borrowed() {
        for xml in [
            "<A><B>text</B></A>",
            "<A>\n\t<B>text</B>\n\t<C/>\n</A>",
            "<A>\n\t<B/>\n</A>",
            "<A><B></B></A>",
            "<A>  <!-- note -->  </A>",
            "<A><![CDATA[ <B>   </B> ]]></A>",
            "<A><!-- <B>   </B> --></A>",
        ] {
            assert!(matches!(preserve(xml), Cow::Borrowed(_)), "{xml:?}");
        }
    }

    #[test]
    fn blank_content_of_a_leaf_is_wrapped() {
        assert_eq!(
            preserve("<A>\n\t<B>   </B>\n\t<C>\t</C>\n</A>"),
            "<A>\n\t<B><![CDATA[   ]]></B>\n\t<C><![CDATA[\t]]></C>\n</A>"
        );
        assert_eq!(
            preserve(r#"<A b="x>y" c='it"s'>  </A>"#),
            r#"<A b="x>y" c='it"s'><![CDATA[  ]]></A>"#
        );
    }

    #[test]
    fn the_scan_finds_a_leaf_and_only_a_leaf() {
        assert!(may_hold_blank_content("<A>   </A>"));
        assert!(may_hold_blank_content(r#"<A b="it's">   </A>"#));
        assert!(!may_hold_blank_content("<A>\n\t<B>x</B>\n</A>"));
        assert!(!may_hold_blank_content("<A>\n\t<B/>\n</A>"));
        assert!(!may_hold_blank_content("<A><?pi x?>   </A>"));
        assert!(!may_hold_blank_content("<A><!-- c -->   </A>"));
        assert!(!may_hold_blank_content(r#"<A b="x>   </A>">z</A>"#));
    }

    #[test]
    fn a_document_the_tokeniser_refuses_is_handed_on_as_it_is() {
        let broken = "<A>   </B>";
        assert!(matches!(preserve(broken), Cow::Borrowed(_)));
    }
}
