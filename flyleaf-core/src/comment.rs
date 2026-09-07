//! The comments a document carries, read and written where they live.
//!
//! TOML has four places a comment can be, and the tree draws all four: above
//! a key or a section header, beside a value, before an array element, and
//! after the last item of the document. `toml_edit` keeps each in a decor,
//! which is the whitespace around a thing: the prefix holds the blank lines,
//! the comment lines and the indentation before it, the suffix the space and
//! the comment after it. Writing a comment means replacing the comment lines
//! and leaving the whitespace as it was, or an edit to a comment would eat
//! the blank line above a section.
//!
//! One thing is not kept: a blank line between two comment lines of the same
//! block. The block is rewritten as one run, so `# a`, a blank, `# b` above
//! a key comes back without the blank once either line is edited. Nothing
//! else about the file moves.

use toml_edit::{Decor, DocumentMut, RawString};

/// The comment lines in a decor's prefix, without their `#` and the
/// whitespace around each.
#[must_use]
pub fn comments_before(decor: &Decor) -> Vec<String> {
    lines_of(decor.prefix())
}

/// The comment in a decor's suffix, which is the one beside a value.
#[must_use]
pub fn comment_beside(decor: &Decor) -> Option<String> {
    lines_of(decor.suffix()).into_iter().next()
}

/// The comment lines after the last item of a document.
#[must_use]
pub fn trailing_comments(doc: &DocumentMut) -> Vec<String> {
    lines_of(Some(doc.trailing()))
}

/// Replace the comment lines in a decor's prefix, keeping the blank lines
/// before them and the indentation after them. No lines removes the comment
/// and leaves the whitespace.
pub fn set_comments_before(decor: &mut Decor, lines: &[String]) {
    let raw = text_of(decor.prefix());
    decor.set_prefix(rewritten(raw, lines));
}

/// Replace the comment in a decor's suffix, keeping the space before it, or
/// remove it.
pub fn set_comment_beside(decor: &mut Decor, comment: Option<&str>) {
    let raw = text_of(decor.suffix());
    let suffix = match comment {
        None => String::new(),
        Some(text) => {
            let before = raw.split('#').next().unwrap_or("");
            // Three spaces where there were none, which is the width the
            // tree's own rows use and enough to read a comment as one.
            let before = if before.trim().is_empty() && !before.is_empty() {
                before
            } else {
                "   "
            };
            format!("{before}{}", hashed(text))
        }
    };
    decor.set_suffix(suffix);
}

/// Replace the comment lines after the last item, keeping the blank lines
/// before them.
pub fn set_trailing_comments(doc: &mut DocumentMut, lines: &[String]) {
    let raw = text_of(Some(doc.trailing()));
    doc.set_trailing(rewritten(raw, lines));
}

fn text_of(raw: Option<&RawString>) -> &str {
    raw.and_then(RawString::as_str).unwrap_or("")
}

fn lines_of(raw: Option<&RawString>) -> Vec<String> {
    text_of(raw)
        .lines()
        .filter_map(|l| l.trim().strip_prefix('#').map(|c| c.trim().to_owned()))
        .collect()
}

/// A comment line as written: `# text`, or a bare `#` for an empty one,
/// since `# ` with nothing after it is a trailing space nobody meant.
fn hashed(text: &str) -> String {
    if text.is_empty() {
        "#".to_owned()
    } else {
        format!("# {text}")
    }
}

/// The prefix with its comment lines replaced.
///
/// A prefix is blank lines, then comment lines, then blank lines again,
/// then the indentation of the thing it precedes, which is whatever follows
/// the last newline. A comment block is as often set off from its key by a
/// blank line as not, so the blank lines on both sides are kept; each new
/// comment line takes the indentation, since a comment above an indented
/// key is indented with it. Where there were no comment lines, every blank
/// line counts as before, so a new comment lands on the line above its key.
fn rewritten(raw: &str, lines: &[String]) -> String {
    let (body, indent) = match raw.rfind('\n') {
        Some(i) => (&raw[..=i], &raw[i + 1..]),
        None => ("", raw),
    };
    let body: Vec<&str> = body.split_inclusive('\n').collect();
    let first_comment = body.iter().position(|l| !l.trim().is_empty());
    let last_comment = body.iter().rposition(|l| !l.trim().is_empty());
    let (before, after) = match (first_comment, last_comment) {
        (Some(first), Some(last)) => (&body[..first], &body[last + 1..]),
        _ => (&body[..], &body[..0]),
    };
    let mut out: String = before.concat();
    for line in lines {
        out.push_str(indent);
        out.push_str(&hashed(line));
        out.push('\n');
    }
    out.push_str(&after.concat());
    out.push_str(indent);
    out
}

#[cfg(test)]
mod tests {
    use super::{
        comment_beside, comments_before, set_comment_beside, set_comments_before,
        set_trailing_comments, trailing_comments,
    };
    use toml_edit::DocumentMut;

    const DOC: &str = "\
# above first

first = 1   # beside first

  # above second, indented
  second = 2


[table]
# in the table
third = 3

# at the end
";

    fn doc() -> DocumentMut {
        DOC.parse().expect("valid TOML")
    }

    /// Every slot reads back what the file has, and nothing else: no blank
    /// lines, no indentation, no `#`.
    #[test]
    fn every_slot_reads_its_comment() {
        let d = doc();
        let t = d.as_table();
        assert_eq!(
            comments_before(t.key("first").unwrap().leaf_decor()),
            ["above first"]
        );
        assert_eq!(
            comment_beside(t["first"].as_value().unwrap().decor()),
            Some("beside first".to_owned())
        );
        assert_eq!(
            comments_before(t.key("second").unwrap().leaf_decor()),
            ["above second, indented"]
        );
        assert_eq!(
            comment_beside(t["second"].as_value().unwrap().decor()),
            None
        );
        assert_eq!(trailing_comments(&d), ["at the end"]);
    }

    /// An edited comment keeps the blank lines above it and the indentation
    /// of the key below it; a removed one leaves both; an added one takes
    /// the indentation of its key. The whole file is compared, so anything
    /// that moved would show.
    #[test]
    fn a_comment_above_is_replaced_and_the_whitespace_stays() {
        let mut d = doc();
        let t = d.as_table_mut();
        set_comments_before(
            t.key_mut("first").unwrap().leaf_decor_mut(),
            &["changed".to_owned(), "and a second line".to_owned()],
        );
        set_comments_before(t.key_mut("second").unwrap().leaf_decor_mut(), &[]);
        set_comments_before(
            t["table"]
                .as_table_mut()
                .unwrap()
                .key_mut("third")
                .unwrap()
                .leaf_decor_mut(),
            &[String::new()],
        );
        assert_eq!(
            d.to_string(),
            "\
# changed
# and a second line

first = 1   # beside first

  second = 2


[table]
#
third = 3

# at the end
"
        );
        assert_eq!(
            comments_before(d.as_table().key("first").unwrap().leaf_decor()),
            ["changed", "and a second line"]
        );
    }

    /// A comment added above a key that had none goes on the line above it,
    /// with its indentation, and the blank line before the key stays where
    /// it was, above the new comment.
    #[test]
    fn a_comment_added_above_takes_the_key_s_indentation() {
        let mut d: DocumentMut = "a = 1\n\n  b = 2\n".parse().unwrap();
        set_comments_before(
            d.as_table_mut().key_mut("b").unwrap().leaf_decor_mut(),
            &["new".to_owned()],
        );
        assert_eq!(d.to_string(), "a = 1\n\n  # new\n  b = 2\n");
    }

    /// A comment beside a value keeps the spacing it had, gets a standard
    /// gap where there was none, and is removed cleanly.
    #[test]
    fn a_comment_beside_is_replaced_added_and_removed() {
        let mut d = doc();
        let t = d.as_table_mut();
        set_comment_beside(
            t["first"].as_value_mut().unwrap().decor_mut(),
            Some("changed"),
        );
        set_comment_beside(t["second"].as_value_mut().unwrap().decor_mut(), Some("new"));
        let written = d.to_string();
        assert!(written.contains("first = 1   # changed\n"), "{written}");
        assert!(written.contains("second = 2   # new\n"), "{written}");

        set_comment_beside(
            d.as_table_mut()["first"]
                .as_value_mut()
                .unwrap()
                .decor_mut(),
            None,
        );
        assert!(d.to_string().contains("first = 1\n"), "{d}");
    }

    /// The comment after the last item is replaced with its blank line
    /// kept, and removed with the blank line kept.
    #[test]
    fn the_trailing_comment_is_replaced_and_removed() {
        let mut d = doc();
        set_trailing_comments(&mut d, &["the end, changed".to_owned()]);
        assert!(
            d.to_string().ends_with("third = 3\n\n# the end, changed\n"),
            "{d}"
        );
        set_trailing_comments(&mut d, &[]);
        assert!(d.to_string().ends_with("third = 3\n\n"), "{d}");
    }

    /// A table header's comment lives in the table's own decor, and is
    /// written there.
    #[test]
    fn a_comment_above_a_header_is_the_table_s() {
        let mut d: DocumentMut = "a = 1\n\n# about t\n[t]\nb = 2\n".parse().unwrap();
        let t = d.as_table_mut()["t"].as_table_mut().unwrap();
        assert_eq!(comments_before(t.decor()), ["about t"]);
        set_comments_before(t.decor_mut(), &["about t, changed".to_owned()]);
        assert_eq!(d.to_string(), "a = 1\n\n# about t, changed\n[t]\nb = 2\n");
    }
}
