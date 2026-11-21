//! The parts of a document whose text is not content: comments, CDATA sections and
//! processing instructions.
//!
//! The scans that run over a whole document before it is read look for a byte or
//! two — a colon that might be a prefix, `&#` that might be a character reference —
//! and only then ask whether what they found is markup at all. Inside one of these
//! sections it is not: the official situation-exchange example carries a whole
//! prefixed DATEX II record inside a comment, and `&#1;` in a CDATA section is four
//! characters of text.

/// The unparsed sections of a document, walked from the front.
///
/// A document holds few sections and, usually, no candidate at all, so this looks
/// for them only when it is asked and remembers where it has got to. Positions have
/// to arrive in increasing order, which is the order they are found in.
pub(super) struct UnparsedText<'a> {
    xml: &'a str,
    /// How far the search has got: everything before this holds no section that has
    /// not been reported.
    searched_to: usize,
    /// Where the section reported last begins and ends.
    section: (usize, usize),
}

impl<'a> UnparsedText<'a> {
    pub(super) fn of(xml: &'a str) -> Self {
        Self {
            xml,
            searched_to: 0,
            section: (0, 0),
        }
    }

    /// Whether `at` falls inside a comment, a CDATA section or a processing
    /// instruction.
    pub(super) fn holds(&mut self, at: usize) -> bool {
        while self.section.1 <= at {
            let Some(section) = self.next_section_before(at) else {
                return false;
            };
            self.section = section;
        }
        self.section.0 <= at
    }

    /// The first section beginning before `at` that has not been accounted for, if
    /// there is one.
    ///
    /// Nothing beyond `at` is searched: a section that begins after it cannot hold it,
    /// and the position asked about next is a better place to search up to than the
    /// end of a document that may be a hundred kilobytes further on.
    fn next_section_before(&mut self, at: usize) -> Option<(usize, usize)> {
        let bytes = self.xml.as_bytes();
        while let Some(found) = memchr::memchr(b'<', &bytes[self.searched_to..at]) {
            let opens_at = self.searched_to + found;
            match length_of_unparsed_text(&self.xml[opens_at..]) {
                Some(length) => {
                    self.searched_to = opens_at + length;
                    return Some((opens_at, opens_at + length));
                }
                None => self.searched_to = opens_at + 1,
            }
        }
        self.searched_to = at;
        None
    }
}

/// The length of the comment, CDATA section or processing instruction beginning
/// here, or `None` when none does. One that is never closed swallows the rest of the
/// document, as it does for a reader.
fn length_of_unparsed_text(from: &str) -> Option<usize> {
    let (opening, closing) = if from.starts_with("<!--") {
        ("<!--", "-->")
    } else if from.starts_with("<![CDATA[") {
        ("<![CDATA[", "]]>")
    } else if from.starts_with("<?") {
        ("<?", "?>")
    } else {
        return None;
    };
    let body = &from[opening.len()..];
    Some(
        body.find(closing)
            .map_or(from.len(), |end| opening.len() + end + closing.len()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_of_section_holds_what_it_encloses() {
        let xml = "<a><!-- x --><![CDATA[ y ]]><?pi z ?><b>w</b></a>";
        let mut unparsed = UnparsedText::of(xml);
        for (text, held) in [("x", true), ("y", true), ("z", true), ("w", false)] {
            let at = xml.find(text).expect("the text is in the document");
            assert_eq!(unparsed.holds(at), held, "{text}");
        }
    }

    #[test]
    fn a_section_that_is_never_closed_holds_the_rest() {
        let xml = "<a><!-- open <b>never</b>";
        assert!(UnparsedText::of(xml).holds(xml.find("never").expect("in the document")));
    }
}
