//! The characters an XML 1.0 document may hold.
//!
//! XML 1.0 admits every Unicode scalar value but the control characters other than
//! tab, line feed and carriage return, and U+FFFE and U+FFFF (`Char`, production 2),
//! whether a character is written as itself or as a character reference (the
//! well-formedness constraint *Legal Character*). A document that holds another is
//! not XML, and a conforming reader refuses the whole of it: a producer that passed
//! a control character from its own source into one would have every consumer drop
//! the delivery. So this crate writes none, and reads none.

use super::unparsed::UnparsedText;

/// The first character in `text` that XML 1.0 does not allow, with the byte offset
/// it stands at, written as itself.
///
/// A Rust string holds no surrogate, so what is left to look for are the control
/// characters, each a single byte, and U+FFFE and U+FFFF, whose encodings both
/// begin with the byte 0xEF.
pub(super) fn first_illegal(text: &str) -> Option<(usize, char)> {
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(found) = bytes[from..]
        .iter()
        .position(|&byte| is_illegal_control(byte) || byte == 0xEF)
    {
        let at = from + found;
        if is_illegal_control(bytes[at]) {
            return Some((at, char::from(bytes[at])));
        }
        match bytes.get(at + 1..at + 3) {
            Some([0xBF, 0xBE]) => return Some((at, '\u{FFFE}')),
            Some([0xBF, 0xBF]) => return Some((at, '\u{FFFF}')),
            _ => from = at + 1,
        }
    }
    None
}

/// The first character reference in the document `xml` that names a character
/// XML 1.0 does not allow, with the byte offset of its `&`.
///
/// A reference is only one in content or in an attribute value: `&#1;` in a comment,
/// a CDATA section or a processing instruction is text. A reference that does not
/// name a code point at all is left to the reader, which refuses it in its own
/// words.
pub(super) fn first_illegal_reference(xml: &str) -> Option<(usize, char)> {
    let mut unparsed = UnparsedText::of(xml);
    memchr::memmem::find_iter(xml.as_bytes(), b"&#").find_map(|at| {
        let named = referenced(&xml[at + 2..])?;
        let character = char::from_u32(named)?;
        (!is_allowed(character) && !unparsed.holds(at)).then_some((at, character))
    })
}

/// The code point a character reference names, read from what follows its `&#`.
fn referenced(reference: &str) -> Option<u32> {
    let (digits, radix) = match reference.strip_prefix('x') {
        Some(hexadecimal) => (hexadecimal, 16),
        None => (reference, 10),
    };
    let digits = &digits[..digits.find(';')?];
    if digits.is_empty() || !digits.chars().all(|digit| digit.is_digit(radix)) {
        return None;
    }
    u32::from_str_radix(digits, radix).ok()
}

/// Whether a byte is one of the control characters XML 1.0 does not allow.
fn is_illegal_control(byte: u8) -> bool {
    byte < 0x20 && !matches!(byte, b'\t' | b'\n' | b'\r')
}

/// Whether XML 1.0 allows the character.
fn is_allowed(character: char) -> bool {
    !matches!(character, '\0'..='\u{8}' | '\u{B}' | '\u{C}' | '\u{E}'..='\u{1F}' | '\u{FFFE}' | '\u{FFFF}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_characters_xml_does_not_allow_are_found_where_they_stand() {
        assert_eq!(first_illegal("a\u{1}b"), Some((1, '\u{1}')));
        assert_eq!(first_illegal("é\u{1f}"), Some((2, '\u{1f}')));
        assert_eq!(first_illegal("a\u{FFFE}"), Some((1, '\u{FFFE}')));
        assert_eq!(first_illegal("\u{FFFF}"), Some((0, '\u{FFFF}')));
        assert_eq!(first_illegal("\t\n\r \u{FEFF}\u{FFFD}\u{F000}\u{10FFFF}"), None);
    }

    #[test]
    fn a_reference_counts_only_where_it_is_markup() {
        assert_eq!(first_illegal_reference("<a>x&#1;</a>"), Some((4, '\u{1}')));
        assert_eq!(first_illegal_reference("<a b=\"&#x1F;\"/>"), Some((6, '\u{1f}')));
        assert_eq!(first_illegal_reference("<a>&#xFFFE;</a>"), Some((3, '\u{FFFE}')));
        assert_eq!(first_illegal_reference("<a><!-- &#1; --><![CDATA[&#1;]]><?p &#1;?></a>"), None);
        assert_eq!(first_illegal_reference("<a>&#9;&#10;&#13;&#xE9;&#65533;</a>"), None);
        assert_eq!(first_illegal_reference("<a>&#;&#x;&#12a;&#99999999999;</a>"), None);
    }
}
