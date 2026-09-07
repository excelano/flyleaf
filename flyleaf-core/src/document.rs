//! A document as a file: the parsed tree, and the three things about its
//! bytes that `toml_edit` reads past and does not write back.
//!
//! Measured on 2026-09-07 against every valid TOML 1.1.0 case in toml-test:
//! a leading byte order mark is dropped, CRLF line endings come back as LF,
//! and a file without a final newline gains one. Each is a fact about the
//! file rather than about the document, so this records them at parse and
//! puts them back at render, and `tests/roundtrip.rs` holds it to that.

use std::fmt;

use toml_edit::{DocumentMut, TomlError};

/// The line ending a file was written with.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Newline {
    /// `\n`, and what a file with no line ending at all is taken to use.
    Lf,
    /// `\r\n`.
    CrLf,
}

/// Why bytes did not become a document.
#[derive(Debug)]
pub enum Error {
    /// The bytes are not UTF-8, which TOML requires them to be.
    NotUtf8(std::str::Utf8Error),
    /// The text is not TOML, with the parser's own account of where.
    Toml(TomlError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUtf8(e) => write!(f, "not UTF-8: {e}"),
            Self::Toml(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

/// A TOML document together with what its file looked like.
///
/// The tree is `toml_edit`'s, reached through [`Document::tree_mut`] for the
/// operations in this crate to work on. What this adds is the byte order
/// mark, the line ending and the final newline, which `toml_edit` does not
/// keep, and a baseline for saying whether anything has been edited.
#[derive(Debug, Clone)]
pub struct Document {
    doc: DocumentMut,
    bom: bool,
    newline: Newline,
    final_newline: bool,
    /// The document as it rendered when it was parsed or last saved, for
    /// [`Document::edited`]. Compared against rather than the source bytes,
    /// because the three facts above make those differ for an unedited file.
    baseline: String,
}

impl Document {
    /// Parse a file's bytes.
    ///
    /// # Errors
    ///
    /// When the bytes are not UTF-8, or the text is not TOML.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let text = std::str::from_utf8(bytes).map_err(Error::NotUtf8)?;
        Self::parse(text)
    }

    /// Parse text, which may begin with a byte order mark.
    ///
    /// # Errors
    ///
    /// When the text is not TOML.
    pub fn parse(text: &str) -> Result<Self, Error> {
        // Recorded, not stripped. `toml_edit` accepts one leading byte order
        // mark and refuses a second, and stripping one here let a file with
        // two through: toml-test's `invalid/encoding/bom-not-at-start-02`
        // found that, and is what holds this line to reading rather than
        // cutting.
        let bom = text.starts_with('\u{feff}');
        // The first line ending decides. A file that mixes them is written
        // back with one of them, which is the only thing a line-ending
        // setting can mean; the corpus test says which cases that touches.
        let newline = match text.find('\n') {
            Some(i) if i > 0 && text.as_bytes()[i - 1] == b'\r' => Newline::CrLf,
            _ => Newline::Lf,
        };
        let final_newline = text.is_empty() || text.ends_with('\n');
        let doc = text.parse::<DocumentMut>().map_err(Error::Toml)?;
        let baseline = doc.to_string();
        Ok(Self {
            doc,
            bom,
            newline,
            final_newline,
            baseline,
        })
    }

    /// The tree, to read.
    #[must_use]
    pub fn tree(&self) -> &DocumentMut {
        &self.doc
    }

    /// The tree, to edit.
    pub fn tree_mut(&mut self) -> &mut DocumentMut {
        &mut self.doc
    }

    /// The line ending the file was written with.
    #[must_use]
    pub fn newline(&self) -> Newline {
        self.newline
    }

    /// Whether the file began with a byte order mark.
    #[must_use]
    pub fn has_bom(&self) -> bool {
        self.bom
    }

    /// What a save writes: the document, with the byte order mark, the line
    /// ending and the final newline the file had.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = self.doc.to_string();
        if !self.final_newline && text.ends_with('\n') {
            // `toml_edit` ends every document with a newline. One that did
            // not have one loses it again here, and only here: a newline
            // inside the document is content.
            text.pop();
        }
        if self.newline == Newline::CrLf {
            text = with_crlf(&text);
        }
        if self.bom {
            text.insert(0, '\u{feff}');
        }
        text
    }

    /// Whether the document differs from what was parsed or last saved.
    #[must_use]
    pub fn edited(&self) -> bool {
        self.doc.to_string() != self.baseline
    }

    /// Record that what the document now holds is what is on disk.
    pub fn mark_saved(&mut self) {
        self.baseline = self.doc.to_string();
    }
}

/// Every bare `\n` as `\r\n`, leaving the ones already paired alone.
///
/// A `\r\n` inside a multi-line string survives the parse as itself, so a
/// blind replacement would double it.
fn with_crlf(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / 40);
    let mut previous = '\0';
    for c in text.chars() {
        if c == '\n' && previous != '\r' {
            out.push('\r');
        }
        out.push(c);
        previous = c;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Document, Error, Newline};

    /// The three things `toml_edit` reads past come back on render, each on
    /// its own and all together. Without the record a document with any of
    /// them would be rewritten the first time it was saved, edited or not.
    #[test]
    fn what_toml_edit_drops_is_put_back() {
        for text in [
            "\u{feff}a = 1\n",
            "a = 1\r\nb = 2\r\n",
            "a = 1",
            "\u{feff}a = 1\r\nb = 2",
            "",
        ] {
            let doc = Document::parse(text).expect("valid TOML");
            assert_eq!(doc.render(), text, "{text:?}");
            assert!(!doc.edited(), "{text:?} is edited before anything happened");
        }
    }

    /// A `\r\n` inside a multi-line string is content, and survives the
    /// parse as itself. Restoring the file's CRLF must not make it `\r\r\n`.
    #[test]
    fn a_crlf_inside_a_string_is_not_doubled() {
        let text = "s = \"\"\"\r\nx\r\n\"\"\"\r\n";
        let doc = Document::parse(text).expect("valid TOML");
        assert_eq!(doc.newline(), Newline::CrLf);
        assert_eq!(doc.render(), text);
    }

    /// Edited is a comparison against the parse, not the bytes, so that a
    /// file with a byte order mark is not edited the moment it is opened;
    /// and saving resets it.
    #[test]
    fn edited_follows_the_document_and_saving_resets_it() {
        let mut doc = Document::parse("\u{feff}a = 1\r\n").expect("valid TOML");
        assert!(!doc.edited());
        doc.tree_mut()["a"] = toml_edit::value(2);
        assert!(doc.edited());
        doc.mark_saved();
        assert!(!doc.edited());
        assert_eq!(doc.render(), "\u{feff}a = 2\r\n");
    }

    /// One byte order mark is a fact about the file; two is not TOML. The
    /// first version of `parse` cut the first off and let `toml_edit` cut the
    /// second, and toml-test caught it.
    #[test]
    fn a_second_byte_order_mark_is_refused() {
        assert!(Document::parse("\u{feff}a = 1\n").is_ok());
        assert!(matches!(
            Document::parse("\u{feff}\u{feff}a = 1\n"),
            Err(Error::Toml(_))
        ));
    }

    /// Bytes that are not UTF-8 and text that is not TOML are two different
    /// refusals, and the second carries the parser's location.
    #[test]
    fn the_two_refusals_are_told_apart() {
        assert!(matches!(
            Document::from_bytes(b"a = \"\xff\"\n"),
            Err(Error::NotUtf8(_))
        ));
        match Document::from_bytes(b"a = \n") {
            Err(Error::Toml(e)) => assert!(e.to_string().contains("line 1"), "{e}"),
            other => panic!("{other:?}"),
        }
    }
}
