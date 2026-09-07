//! A document as a file: the parsed tree, the three things about its bytes
//! that `toml_edit` reads past and does not write back, and its history.
//!
//! Measured on 2026-09-07 against every valid TOML 1.1.0 case in toml-test:
//! a leading byte order mark is dropped, CRLF line endings come back as LF,
//! and a file without a final newline gains one. Each is a fact about the
//! file rather than about the document, so this records them at parse and
//! puts them back at render, and `tests/roundtrip.rs` holds it to that.
//!
//! The history is the document's rendered text, one copy per step. A step
//! is what changed between two calls to [`Document::record`], and calls
//! that name the same row coalesce, so typing into a field is one step and
//! not one per keystroke. Undo parses the previous text back, which loses
//! nothing, the text being the whole of the document. A copy of the file per
//! step is the cost, about 100 KB for a `Cargo.lock`, which is cheap beside
//! what the window holds for the same document.

use std::fmt;

use std::ops::Range;

use toml_edit::{DocumentMut, Item, Table, TomlError, Value};

/// The line ending a file was written with.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Newline {
    /// `\n`, and what a file with no line ending at all is taken to use.
    Lf,
    /// `\r\n`.
    CrLf,
}

/// Why a file did not become a document, or a document a file.
#[derive(Debug)]
pub enum Error {
    /// The bytes are not UTF-8, which TOML requires them to be.
    NotUtf8(std::str::Utf8Error),
    /// The text is not TOML, with the parser's own account of where.
    Toml(TomlError),
    /// The file could not be read or written.
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUtf8(e) => write!(f, "not UTF-8: {e}"),
            Self::Toml(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
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
    /// The document as it rendered at the last [`Document::record`], which
    /// is what the next change is measured against.
    current: String,
    /// The text before each step, newest last.
    undo: Vec<String>,
    /// The text undone, newest last, emptied by any new change.
    redo: Vec<String>,
    /// The row the last step was made in, which is what a further change
    /// to the same row coalesces with.
    group: Option<Vec<String>>,
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
            current: baseline.clone(),
            baseline,
            undo: Vec::new(),
            redo: Vec::new(),
            group: None,
        })
    }

    /// Read and parse a file.
    ///
    /// # Errors
    ///
    /// When the file cannot be read, is not UTF-8, or is not TOML.
    #[cfg(feature = "fs")]
    pub fn from_path(path: &std::path::Path) -> Result<Self, Error> {
        Self::from_bytes(&std::fs::read(path)?)
    }

    /// Write what [`Document::render`] gives to a file, and mark the
    /// document saved.
    ///
    /// Written beside the file and renamed over it, so that a failure at any
    /// point leaves the original as it was and never a file half written.
    /// The rename is what makes it one step, and it needs the two on one
    /// file system, which a sibling is. A platform whose sandbox refuses a
    /// sibling, which macOS's does for a file a dialog granted, needs its
    /// own arm here; `PROMPT.md` carries that under Phase 3.
    ///
    /// # Errors
    ///
    /// When the sibling cannot be created, written, or renamed over the
    /// file. The document is not marked saved then.
    #[cfg(feature = "fs")]
    pub fn save_to(&mut self, path: &std::path::Path) -> Result<(), Error> {
        use std::io::Write as _;
        let beside = path.parent().filter(|p| !p.as_os_str().is_empty());
        let mut staged = match beside {
            Some(dir) => tempfile::NamedTempFile::new_in(dir)?,
            None => tempfile::NamedTempFile::new_in(".")?,
        };
        staged.write_all(self.render().as_bytes())?;
        staged.as_file().sync_all()?;
        staged.persist(path).map_err(|e| e.error)?;
        self.mark_saved();
        Ok(())
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

    /// Record whatever changed since the last call as a step, and say
    /// whether anything did.
    ///
    /// `group` names the row the change was made in, where there is one.
    /// A change in the same row as the step before joins that step rather
    /// than starting another, so a word typed into a field undoes as a word;
    /// a change with no row, or in another row, is a step of its own. Any
    /// change empties the redo stack, since what was undone no longer
    /// follows from what is there.
    pub fn record(&mut self, group: Option<&[String]>) -> bool {
        let now = self.doc.to_string();
        if now == self.current {
            return false;
        }
        let same_row = group.is_some() && self.group.as_deref() == group;
        if !same_row {
            let before = std::mem::take(&mut self.current);
            self.undo.push(before);
            self.group = group.map(<[String]>::to_vec);
        }
        self.current = now;
        self.redo.clear();
        true
    }

    /// Whether there is a step to undo.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether there is a step to redo.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Put the document back as it was before the last step.
    ///
    /// Anything changed since the last [`Document::record`] is recorded
    /// first, so that it can be redone rather than lost.
    pub fn undo(&mut self) -> bool {
        self.record(None);
        let Some(before) = self.undo.pop() else {
            return false;
        };
        let now = std::mem::replace(&mut self.current, before);
        self.redo.push(now);
        self.restore();
        true
    }

    /// Put back the last step undone.
    pub fn redo(&mut self) -> bool {
        let Some(after) = self.redo.pop() else {
            return false;
        };
        let now = std::mem::replace(&mut self.current, after);
        self.undo.push(now);
        self.restore();
        true
    }

    /// The lines of the rendered text that the item at a path occupies,
    /// counted from zero, the end exclusive: a row from its key to the end
    /// of its value, a table or an array-of-tables element its header line.
    /// `None` for a path that names nothing.
    ///
    /// The tree keeps no positions once it can be edited, so this parses the
    /// rendered text again, which does. That is a full parse per call, and a
    /// caller asks only when the selection or the document has changed.
    #[must_use]
    pub fn lines_of(&self, path: &[String]) -> Option<Range<usize>> {
        let text = self.doc.to_string();
        let parsed = toml_edit::Document::parse(text.as_str()).ok()?;
        let span = span_in_table(parsed.as_table(), path)?;
        let line_at = |offset: usize| text[..offset.min(text.len())].matches('\n').count();
        Some(line_at(span.start)..line_at(span.end.saturating_sub(1)) + 1)
    }

    /// The tree as `current` says, which parses because it was rendered
    /// from a tree; a step is never a row's own, so the next change starts
    /// one.
    fn restore(&mut self) {
        self.doc = self
            .current
            .parse::<DocumentMut>()
            .expect("a rendered document parses");
        self.group = None;
    }
}

/// The span of the item at a path under a table: the key and the item
/// together where the path ends here, or whatever is further down.
fn span_in_table(t: &Table, path: &[String]) -> Option<Range<usize>> {
    let (head, rest) = path.split_first()?;
    let (key, item) = t.get_key_value(head)?;
    if rest.is_empty() {
        return join(key.span(), item.span());
    }
    match item {
        Item::Table(inner) => span_in_table(inner, rest),
        Item::ArrayOfTables(a) => {
            let (index, rest) = rest.split_first()?;
            let element = a.get(indexed(index)?)?;
            if rest.is_empty() {
                element.span()
            } else {
                span_in_table(element, rest)
            }
        }
        Item::Value(v) => span_in_value(v, rest),
        Item::None => None,
    }
}

/// The same under a value, which is an inline table or an array if the path
/// goes on.
fn span_in_value(v: &Value, path: &[String]) -> Option<Range<usize>> {
    let (head, rest) = path.split_first()?;
    match v {
        Value::InlineTable(t) => {
            let (key, inner) = t.get_key_value(head)?;
            if rest.is_empty() {
                join(key.span(), inner.span())
            } else {
                span_in_value(inner.as_value()?, rest)
            }
        }
        Value::Array(a) => {
            let element = a.get(indexed(head)?)?;
            if rest.is_empty() {
                element.span()
            } else {
                span_in_value(element, rest)
            }
        }
        _ => None,
    }
}

/// The index an element's path segment names, written `[3]` as the tree
/// labels it.
fn indexed(segment: &str) -> Option<usize> {
    segment.strip_prefix('[')?.strip_suffix(']')?.parse().ok()
}

/// One span from the start of the first to the end of the last.
fn join(a: Option<Range<usize>>, b: Option<Range<usize>>) -> Option<Range<usize>> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.start.min(b.start)..a.end.max(b.end)),
        (Some(a), None) | (None, Some(a)) => Some(a),
        (None, None) => None,
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

    /// Typing into one row is one step, another row is another, and undo
    /// and redo walk them in order. Without coalescing every keystroke would
    /// be a step and undo would take a word back one letter at a time.
    #[test]
    fn changes_in_one_row_are_one_step() {
        let mut doc = Document::parse("a = \"\"\nb = 0\n").expect("valid TOML");
        let a = vec!["a".to_owned()];
        let b = vec!["b".to_owned()];
        for text in ["x", "xy", "xyz"] {
            doc.tree_mut()["a"] = toml_edit::value(text);
            assert!(doc.record(Some(&a)));
        }
        doc.tree_mut()["b"] = toml_edit::value(1);
        assert!(doc.record(Some(&b)));
        assert!(!doc.record(Some(&b)), "nothing changed");
        assert_eq!(doc.render(), "a = \"xyz\"\nb = 1\n");

        assert!(doc.undo());
        assert_eq!(doc.render(), "a = \"xyz\"\nb = 0\n");
        assert!(doc.undo());
        assert_eq!(
            doc.render(),
            "a = \"\"\nb = 0\n",
            "the word came back whole"
        );
        assert!(!doc.undo(), "nothing left to undo");

        assert!(doc.redo());
        assert_eq!(doc.render(), "a = \"xyz\"\nb = 0\n");
        assert!(doc.redo());
        assert_eq!(doc.render(), "a = \"xyz\"\nb = 1\n");
        assert!(!doc.redo());
    }

    /// A change after an undo is a new branch: what was undone cannot be
    /// redone over it. And a change nobody recorded before pressing undo is
    /// recorded then, so it is undone rather than lost.
    #[test]
    fn a_change_after_an_undo_ends_the_redo_and_an_unrecorded_one_is_kept() {
        let mut doc = Document::parse("a = 1\n").expect("valid TOML");
        doc.tree_mut()["a"] = toml_edit::value(2);
        doc.record(None);
        assert!(doc.undo());
        assert!(doc.can_redo());
        doc.tree_mut()["a"] = toml_edit::value(3);
        doc.record(None);
        assert!(!doc.can_redo(), "a new change ended the branch");

        doc.tree_mut()["a"] = toml_edit::value(4);
        assert!(doc.undo(), "the unrecorded change is a step");
        assert_eq!(doc.render(), "a = 3\n");
        assert!(doc.redo());
        assert_eq!(doc.render(), "a = 4\n");
    }

    /// Undo puts back the text, comments and layout included, and leaves
    /// the file facts and the saved baseline alone: an undo past the save is
    /// edited, an undo back to it is not.
    #[test]
    fn undo_restores_the_text_and_respects_the_baseline() {
        let text = "\u{feff}# above\r\na = 1   # beside\r\n";
        let mut doc = Document::parse(text).expect("valid TOML");
        doc.tree_mut()["a"] = toml_edit::value(2);
        doc.record(None);
        doc.mark_saved();
        doc.tree_mut()["a"] = toml_edit::value(3);
        doc.record(None);
        assert!(doc.edited());
        assert!(doc.undo());
        assert!(!doc.edited(), "back at what was saved");
        assert!(doc.undo());
        assert!(doc.edited(), "before what was saved");
        assert_eq!(doc.render(), text);
    }

    /// Every shape of path lands on the lines it names: a root row, a row in
    /// a table, a table's header, an array element, an array-of-tables
    /// element's header, a key inside an inline table, and a value that runs
    /// over several lines, whose range is all of them. A path naming nothing
    /// is `None` rather than a wrong line.
    #[test]
    fn every_kind_of_path_lands_on_its_lines() {
        let text = "\
first = 1
[types]
count = 44
list = [
  1,
  2,
]
inline = { a = 1, b = 2 }
[[runs]]
id = 1
[[runs]]
id = 2
";
        let doc = Document::parse(text).expect("valid TOML");
        let lines = |parts: &[&str]| {
            let path: Vec<String> = parts.iter().map(|p| (*p).to_owned()).collect();
            doc.lines_of(&path)
        };
        assert_eq!(lines(&["first"]), Some(0..1));
        assert_eq!(lines(&["types"]), Some(1..2));
        assert_eq!(lines(&["types", "count"]), Some(2..3));
        assert_eq!(lines(&["types", "list"]), Some(3..7));
        assert_eq!(lines(&["types", "list", "[1]"]), Some(5..6));
        assert_eq!(lines(&["types", "inline", "b"]), Some(7..8));
        assert_eq!(lines(&["runs", "[1]"]), Some(10..11));
        assert_eq!(lines(&["runs", "[1]", "id"]), Some(11..12));
        assert_eq!(lines(&["absent"]), None);
        assert_eq!(lines(&["types", "list", "[9]"]), None);
        assert_eq!(lines(&[]), None);
    }

    /// A document saved and read back is the same bytes, byte order mark and
    /// line endings included, and saving is what clears the edited mark. A
    /// save that cannot happen leaves the file as it was and the mark set.
    #[cfg(feature = "fs")]
    #[test]
    fn a_save_writes_the_render_and_a_failed_one_writes_nothing() {
        let dir = std::env::temp_dir().join(format!("flyleaf-core-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("doc.toml");
        std::fs::write(&path, "\u{feff}a = 1\r\n").unwrap();

        let mut doc = Document::from_path(&path).expect("reads");
        doc.tree_mut()["a"] = toml_edit::value(2);
        assert!(doc.edited());
        doc.save_to(&path).expect("saves");
        assert!(!doc.edited());
        assert_eq!(
            std::fs::read(&path).unwrap(),
            "\u{feff}a = 2\r\n".as_bytes()
        );
        assert_eq!(
            std::fs::read_dir(&dir).unwrap().count(),
            1,
            "the staged file is gone"
        );

        doc.tree_mut()["a"] = toml_edit::value(3);
        let nowhere = dir.join("missing").join("doc.toml");
        assert!(matches!(doc.save_to(&nowhere), Err(Error::Io(_))));
        assert!(doc.edited(), "not marked saved");
        assert_eq!(
            std::fs::read(&path).unwrap(),
            "\u{feff}a = 2\r\n".as_bytes()
        );
        std::fs::remove_dir_all(&dir).unwrap();
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
