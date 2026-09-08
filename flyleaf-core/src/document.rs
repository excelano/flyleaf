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
    /// A rename needs only the directory to be writable, so two things an
    /// in-place write would do for free are done here on purpose: a file
    /// marked read-only is refused rather than replaced, and the file keeps
    /// the permissions it had rather than the staged file's. Both were
    /// found by hand on 2026-09-07, when a read-only fixture saved without
    /// a word and came back mode 0600.
    ///
    /// # Errors
    ///
    /// When the file is read-only, or the sibling cannot be created,
    /// written, or renamed over the file. The document is not marked saved
    /// then.
    #[cfg(feature = "fs")]
    pub fn save_to(&mut self, path: &std::path::Path) -> Result<(), Error> {
        use std::io::Write as _;
        let dir = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."));
        let original = match std::fs::metadata(path) {
            Ok(m) => Some(m),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        if original
            .as_ref()
            .is_some_and(|m| m.permissions().readonly())
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "the file is read-only",
            )
            .into());
        }

        // On macOS a file that already exists is replaced through the
        // platform's own call rather than through a sibling, because under
        // the App Sandbox the grant a person gives by choosing a file covers
        // the file and not its directory, and a temporary sibling stops with
        // *Operation not permitted*. A new file has no such grant problem:
        // the save panel's grant covers the name chosen, and it is written
        // directly, there being nothing to lose if the write fails.
        #[cfg(target_os = "macos")]
        if original.is_some() {
            macos::replace(path, self.render().as_bytes())?;
            self.mark_saved();
            return Ok(());
        }

        // `mut` for the permissions call below, which only Unix makes; the
        // Windows build is checked with warnings as errors and would refuse
        // the unused `mut` there.
        #[cfg_attr(not(unix), allow(unused_mut))]
        let mut builder = tempfile::Builder::new();
        // A temporary file is made private, which is right for one and
        // wrong for a document somebody will keep: a new file gets what
        // the umask gives any new file, and an existing one gets its own
        // permissions back below.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            builder.permissions(std::fs::Permissions::from_mode(0o666));
        }
        let mut staged = builder.tempfile_in(dir)?;
        staged.write_all(self.render().as_bytes())?;
        staged.as_file().sync_all()?;
        if let Some(original) = original {
            staged.as_file().set_permissions(original.permissions())?;
        }
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

    /// The defect this catches is the rewrite waiting beside the file.
    ///
    /// On macOS that is what makes Save fail under the App Sandbox: the grant
    /// a person gives through the open panel covers the file they chose and
    /// not the directory holding it, so a randomly-named sibling stops with
    /// *Operation not permitted*, measured in slipcase-desktop. A test cannot
    /// enter a sandbox, so it asserts the property the sandbox refuses: after
    /// a save, nothing but the file is in the file's directory, which a
    /// sibling that failed to land, or one still waiting, would break. It runs
    /// on the Apple silicon workflow, since no machine here is a Mac.
    #[cfg(all(feature = "fs", target_os = "macos"))]
    #[test]
    fn a_save_leaves_nothing_beside_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("only.toml");
        std::fs::write(&path, "a = 1\n").unwrap();
        let mut doc = Document::from_path(&path).unwrap();
        doc.tree_mut()["a"] = toml_edit::value(2);
        doc.save_to(&path).unwrap();
        let beside: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(beside, vec![std::ffi::OsString::from("only.toml")]);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "a = 2\n");
    }
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

    /// A saved file keeps the permissions it had, and a read-only file is
    /// refused rather than replaced. A rename needs only the directory to be
    /// writable, so without these a read-only fixture saved without a word
    /// and came back mode 0600, which is how they were found.
    #[cfg(all(feature = "fs", unix))]
    #[test]
    fn a_save_keeps_the_mode_and_refuses_a_read_only_file() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = std::env::temp_dir().join(format!("flyleaf-core-mode-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("doc.toml");
        std::fs::write(&path, "a = 1\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();

        let mut doc = Document::from_path(&path).expect("reads");
        doc.tree_mut()["a"] = toml_edit::value(2);
        doc.save_to(&path).expect("saves");
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o640, "the mode the file had");

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o440)).unwrap();
        doc.tree_mut()["a"] = toml_edit::value(3);
        match doc.save_to(&path) {
            Err(Error::Io(e)) => assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied),
            other => panic!("{other:?}"),
        }
        assert!(doc.edited(), "not marked saved");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "a = 2\n");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
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

/// Replacing a file the way a sandboxed application is allowed to.
///
/// slipcase-desktop's `staging.rs` is where both calls were measured, and the
/// two findings there decide the shape of this: the rewrite waits in the
/// directory macOS provides for replacements, asked for with the file's own
/// URL so that it lands on the file's volume, because a directory under
/// `TMPDIR` fails the replacement with `EXDEV` for any file not on the boot
/// volume; and it lands with `replaceItemAtURL:`, which keeps the original's
/// permissions and extended attributes, where a rename into the file's
/// directory is a write a sandboxed application has no grant for.
#[cfg(all(feature = "fs", target_os = "macos"))]
mod macos {
    use std::path::{Path, PathBuf};

    use objc2_foundation::{
        NSFileManager, NSFileManagerItemReplacementOptions, NSSearchPathDirectory,
        NSSearchPathDomainMask, NSString, NSURL,
    };

    /// Write `bytes` over the file at `path`, whole or not at all.
    pub(super) fn replace(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        // Resolved first, for the reason a rename would resolve it: a file
        // reached through a symbolic link should have the file replaced and
        // not the link, and the volume that matters is the file's.
        let original = std::fs::canonicalize(path)?;
        let scratch = Scratch::on_the_volume_holding(&original)?;
        let name = original.file_name().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{} names no file", original.display()),
            )
        })?;
        let staged = scratch.path().join(name);
        std::fs::write(&staged, bytes)?;
        std::fs::File::open(&staged)?.sync_all()?;

        NSFileManager::defaultManager()
            .replaceItemAtURL_withItemAtURL_backupItemName_options_resultingItemURL_error(
                &url_for(&original),
                &url_for(&staged),
                None,
                NSFileManagerItemReplacementOptions::empty(),
                None,
            )
            .map_err(|e| {
                std::io::Error::other(format!(
                    "cannot replace {}: {}{}",
                    original.display(),
                    e.localizedDescription(),
                    because_of(&e)
                ))
            })?;
        // After the replacement, which has moved the staged file out.
        drop(scratch);
        Ok(())
    }

    /// The directory macOS made for one replacement, removed when it is over.
    struct Scratch(PathBuf);

    impl Scratch {
        /// `NSItemReplacementDirectory` in the user domain, `appropriateForURL:`
        /// being what decides where the directory lands.
        fn on_the_volume_holding(original: &Path) -> std::io::Result<Self> {
            let url = NSFileManager::defaultManager()
                .URLForDirectory_inDomain_appropriateForURL_create_error(
                    NSSearchPathDirectory::ItemReplacementDirectory,
                    NSSearchPathDomainMask::UserDomainMask,
                    Some(&url_for(original)),
                    true,
                )
                .map_err(|e| {
                    std::io::Error::other(format!(
                        "nowhere to rewrite {}: {}",
                        original.display(),
                        e.localizedDescription()
                    ))
                })?;
            let path = url.path().ok_or_else(|| {
                std::io::Error::other("macOS named a replacement directory with no path")
            })?;
            Ok(Self(PathBuf::from(path.to_string())))
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            // The whole tree: a replacement that succeeded left this empty, and
            // one that failed left the staged file in it.
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// The errno under Cocoa's sentence, if it said, because
    /// `localizedDescription` is written for a dialog and hides the fact worth
    /// having; `EXDEV` is the one this module exists to avoid.
    fn because_of(error: &objc2_foundation::NSError) -> String {
        use std::fmt::Write as _;
        error
            .underlyingErrors()
            .iter()
            .fold(String::new(), |mut so_far, under| {
                let _ = write!(
                    so_far,
                    " ({} {})",
                    under.domain(),
                    under.localizedDescription()
                );
                so_far
            })
    }

    fn url_for(path: &Path) -> objc2::rc::Retained<NSURL> {
        NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()))
    }
}
