//! The document behind Tommy Flyleaf: a TOML file edited as a tree, with its
//! comments, key order and formatting kept for everything that was not edited.
//!
//! No user interface. Everything here is an operation on a `toml_edit`
//! document that the widget in `flyleaf` asks for and never performs itself,
//! so that there is one place a change goes through.
//
// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]

pub use toml_edit;

mod array;
mod document;
mod kind;

pub use array::{convert_element, push_element, remove_element, set_element};
pub use document::{Document, Error, Newline};
pub use kind::{convert, Kind};

use toml_edit::{InlineTable, Item, Key, Table, Value};

/// Add a key to a table.
///
/// Refuses an empty name and one already there: inserting over an existing key
/// would replace it, and losing a value to a name collision is not what
/// pressing Add asks for.
pub fn add_key(t: &mut Table, name: &str, kind: Kind) -> bool {
    if name.is_empty() || t.contains_key(name) {
        return false;
    }
    t.insert(name, kind.item());
    true
}

/// Remove a key, and everything under it where it is a table.
pub fn remove_key(t: &mut Table, name: &str) -> bool {
    t.remove(name).is_some()
}

/// Rename a key, keeping its place in the document and the decor it carried.
///
/// `toml_edit` has no rename. Removing and re-inserting would put the key at the
/// end, and authoring order carries intent, so every entry is taken out in
/// order and put back with the renamed one rebuilt under its new name.
/// `remove_entry` hands back the `Key` itself, so the comments and whitespace
/// attached to it come along.
pub fn rename_key(t: &mut Table, from: &str, to: &str) -> bool {
    if to.is_empty() || from == to || !t.contains_key(from) || t.contains_key(to) {
        return false;
    }

    let names: Vec<String> = t.iter().map(|(k, _)| k.to_owned()).collect();
    let mut taken: Vec<(Key, Item)> = Vec::with_capacity(names.len());
    for name in &names {
        if let Some(entry) = t.remove_entry(name) {
            taken.push(entry);
        }
    }

    for (key, item) in taken {
        if key.get() == from {
            let renamed = Key::new(to)
                .with_leaf_decor(key.leaf_decor().clone())
                .with_dotted_decor(key.dotted_decor().clone());
            t.insert_formatted(&renamed, item);
        } else {
            t.insert_formatted(&key, item);
        }
    }
    true
}

/// Add a key to an inline table.
///
/// The same refusals as [`add_key`], and one more: a kind an inline table
/// cannot hold.
pub fn add_inline_key(t: &mut InlineTable, name: &str, kind: Kind) -> bool {
    if name.is_empty() || t.contains_key(name) {
        return false;
    }
    match kind.as_value() {
        Some(value) => {
            t.insert(name, value);
            true
        }
        None => false,
    }
}

/// Remove a key from an inline table.
pub fn remove_inline_key(t: &mut InlineTable, name: &str) -> bool {
    t.remove(name).is_some()
}

/// Rename a key in an inline table, keeping its place and its decor.
///
/// The same rebuild [`rename_key`] does, for the same reason: there is no
/// rename, and re-inserting would move the key to the end of a line somebody
/// wrote in an order they chose.
pub fn rename_inline_key(t: &mut InlineTable, from: &str, to: &str) -> bool {
    if to.is_empty() || from == to || !t.contains_key(from) || t.contains_key(to) {
        return false;
    }

    let names: Vec<String> = t.iter().map(|(k, _)| k.to_owned()).collect();
    let mut taken: Vec<(Key, Value)> = Vec::with_capacity(names.len());
    for name in &names {
        if let Some(entry) = t.remove_entry(name) {
            taken.push(entry);
        }
    }

    for (key, value) in taken {
        if key.get() == from {
            let renamed = Key::new(to)
                .with_leaf_decor(key.leaf_decor().clone())
                .with_dotted_decor(key.dotted_decor().clone());
            t.insert_formatted(&renamed, value);
        } else {
            t.insert_formatted(&key, value);
        }
    }
    true
}

/// Change a key's value to another kind, where it reads as one, keeping its
/// place and its decor.
///
/// A value becomes another value through [`convert`]. A table becomes an
/// inline table and an inline table becomes a table through `toml_edit`'s
/// own conversions, which is the one change here that is about layout rather
/// than type: `[owner]` and `owner = { ... }` hold the same thing, and which
/// one a file uses is a decision this makes explicit. Refuses what would be a
/// guess, and an array of tables, which is neither one table nor a value.
pub fn convert_key(t: &mut Table, name: &str, to: Kind) -> bool {
    let Some(item) = t.get_mut(name) else {
        return false;
    };
    match (item, to) {
        (Item::Value(v), to) if to != Kind::Table => match convert(v, to) {
            Some(new) => {
                set_value(v, new);
                true
            }
            None => false,
        },
        (Item::Value(Value::InlineTable(_)), Kind::Table) | (Item::Table(_), Kind::InlineTable) => {
            relayout(t, name)
        }
        _ => false,
    }
}

/// A table as an inline table or back, in the same place with the same key.
///
/// The same rebuild [`rename_key`] does, because the entry has to come out
/// to be converted and `insert` would put it back at the end.
fn relayout(t: &mut Table, name: &str) -> bool {
    let names: Vec<String> = t.iter().map(|(k, _)| k.to_owned()).collect();
    let mut taken: Vec<(Key, Item)> = Vec::with_capacity(names.len());
    for n in &names {
        if let Some(entry) = t.remove_entry(n) {
            taken.push(entry);
        }
    }
    for (key, item) in taken {
        if key.get() != name {
            t.insert_formatted(&key, item);
            continue;
        }
        // A fresh key for the new layout: a header's key carries no space
        // after it and would render `owner= {`, and a key-value line's would
        // render `[owner ]`. What is kept is the comment above: a table
        // holds it in its own decor, a key in its prefix, and the layout
        // change carries it from the one to the other.
        let (key, item) = match item {
            Item::Table(table) => {
                let mut key = Key::new(name);
                if let Some(above) = table.decor().prefix().cloned() {
                    key.leaf_decor_mut().set_prefix(above);
                }
                (
                    key,
                    Item::Value(Value::InlineTable(table.into_inline_table())),
                )
            }
            Item::Value(Value::InlineTable(inline)) => {
                let mut table = inline.into_table();
                if let Some(above) = key.leaf_decor().prefix().cloned() {
                    table.decor_mut().set_prefix(above);
                }
                (Key::new(name), Item::Table(table))
            }
            other => (key, other),
        };
        t.insert_formatted(&key, item);
    }
    true
}

/// Change a key's value in an inline table to another kind, where it reads
/// as one. A table cannot be inside an inline table, so that is refused.
pub fn convert_inline_key(t: &mut InlineTable, name: &str, to: Kind) -> bool {
    if to == Kind::Table {
        return false;
    }
    let Some(v) = t.get_mut(name) else {
        return false;
    };
    match convert(v, to) {
        Some(new) => {
            set_value(v, new);
            true
        }
        None => false,
    }
}

/// Change a value, keeping the decor it was written with.
///
/// Dropping a new `Item` over an old one discards its decor, which is the
/// whitespace and the comments attached to it, so the value is assigned into
/// and its decor put back afterwards.
pub fn set_value(slot: &mut Value, new: Value) {
    let decor = slot.decor().clone();
    *slot = new;
    *slot.decor_mut() = decor;
}

#[cfg(test)]
mod structure_tests {
    use super::{add_key, remove_key, rename_key, Kind};
    use toml_edit::DocumentMut;

    const DOC: &str = "\
# above first
first = \"one\"   # beside first
second = 2

[third]
inner = true
";

    fn doc() -> DocumentMut {
        DOC.parse().expect("valid TOML")
    }

    fn keys(d: &DocumentMut) -> Vec<String> {
        d.as_table().iter().map(|(k, _)| k.to_owned()).collect()
    }

    /// Authoring order carries intent. A rename is where `toml_edit` would
    /// quietly move the key to the end, having no rename of its own.
    #[test]
    fn a_rename_keeps_its_place_and_its_comments() {
        let mut d = doc();
        assert!(rename_key(d.as_table_mut(), "first", "primary"));

        assert_eq!(keys(&d), ["primary", "second", "third"]);

        let written = d.to_string();
        assert!(written.contains("# above first"), "{written}");
        assert!(written.contains("# beside first"), "{written}");
        assert!(written.contains("primary = \"one\""), "{written}");
        assert!(!written.contains("first ="), "{written}");
    }

    /// A table renamed keeps its place too, and its contents come with it.
    #[test]
    fn a_table_can_be_renamed() {
        let mut d = doc();
        assert!(rename_key(d.as_table_mut(), "third", "provenance"));

        assert_eq!(keys(&d), ["first", "second", "provenance"]);
        assert!(d.to_string().contains("inner = true"));
    }

    /// A rename that would land on a name already there is refused rather than
    /// replacing it: losing a value to a collision is not what renaming asks
    /// for.
    #[test]
    fn a_rename_onto_an_existing_key_is_refused() {
        let mut d = doc();
        assert!(!rename_key(d.as_table_mut(), "first", "second"));
        assert!(!rename_key(d.as_table_mut(), "first", ""));
        assert!(!rename_key(d.as_table_mut(), "absent", "anything"));

        assert_eq!(keys(&d), ["first", "second", "third"]);
        assert!(d.to_string().contains("second = 2"));
    }

    /// A key added to a document that already has tables stays at the root.
    ///
    /// It goes last in the map, after the table, and `toml_edit` still writes it
    /// above the table's header. That is the difference between map order and
    /// document order, and it is the one that matters: a bare key written after
    /// `[third]` would be a key inside `third` rather than a key of the
    /// document, which is a different document.
    #[test]
    fn a_key_added_to_a_document_with_tables_stays_at_the_root() {
        let mut d = doc();
        assert!(add_key(d.as_table_mut(), "author", Kind::Text));
        assert_eq!(keys(&d), ["first", "second", "third", "author"]);

        let written = d.to_string();
        assert!(written.contains("author = \"\""), "{written}");
        assert!(
            written.find("author").unwrap() < written.find("[third]").unwrap(),
            "an added key must not fall inside the last table: {written}"
        );

        // Read back rather than trusted: this is where it would go wrong.
        let back: DocumentMut = written.parse().expect("still parses");
        assert!(back.as_table().contains_key("author"));
        assert!(!back["third"]
            .as_table()
            .expect("a table")
            .contains_key("author"));
    }

    #[test]
    fn a_key_already_there_is_not_added_over() {
        let mut d = doc();
        assert!(!add_key(d.as_table_mut(), "first", Kind::Integer));
        assert!(!add_key(d.as_table_mut(), "", Kind::Integer));
        assert!(d.to_string().contains("first = \"one\""));
    }

    /// Every kind a picker offers produces a document that still parses.
    #[test]
    fn every_kind_of_new_key_is_valid_toml() {
        for kind in Kind::ALL {
            let mut d = doc();
            assert!(add_key(d.as_table_mut(), "added", kind), "{kind:?}");
            let written = d.to_string();
            written
                .parse::<DocumentMut>()
                .unwrap_or_else(|e| panic!("{kind:?} wrote something unparseable: {e}\n{written}"));
        }
    }

    #[test]
    fn removing_a_table_takes_what_is_under_it() {
        let mut d = doc();
        assert!(remove_key(d.as_table_mut(), "third"));
        assert_eq!(keys(&d), ["first", "second"]);
        assert!(!d.to_string().contains("inner"));
        assert!(!remove_key(d.as_table_mut(), "third"));
    }
}

#[cfg(test)]
mod inline_tests {
    use super::{add_inline_key, remove_inline_key, rename_inline_key, Kind};
    use toml_edit::DocumentMut;

    fn doc() -> DocumentMut {
        "owner = { name = \"D. Anderson\", team = \"consulting\" }\n"
            .parse()
            .expect("valid TOML")
    }

    fn owner(d: &mut DocumentMut) -> &mut toml_edit::InlineTable {
        d["owner"].as_inline_table_mut().expect("an inline table")
    }

    /// The bug this closes: the buttons were drawn inside an inline table and
    /// the change was thrown away, so `owner` could be removed and `owner.name`
    /// could not.
    #[test]
    fn a_key_inside_an_inline_table_can_be_removed() {
        let mut d = doc();
        assert!(remove_inline_key(owner(&mut d), "name"));
        assert!(!remove_inline_key(owner(&mut d), "name"));

        let written = d.to_string();
        assert!(!written.contains("name"), "{written}");
        assert!(written.contains("team = \"consulting\""), "{written}");
    }

    /// Renamed in place, not moved to the end of a line somebody wrote in an
    /// order they chose.
    #[test]
    fn a_key_inside_an_inline_table_keeps_its_place_when_renamed() {
        let mut d = doc();
        assert!(rename_inline_key(owner(&mut d), "name", "who"));

        let written = d.to_string();
        assert!(
            written.find("who").unwrap() < written.find("team").unwrap(),
            "{written}"
        );
        assert!(written.contains("who = \"D. Anderson\""), "{written}");

        // Refused for the same reasons as anywhere else.
        assert!(!rename_inline_key(owner(&mut d), "who", "team"));
        assert!(!rename_inline_key(owner(&mut d), "who", ""));
    }

    /// An inline table holds values, so a table is not among the kinds offered
    /// and is refused if it arrives anyway.
    #[test]
    fn an_inline_table_takes_values_and_not_tables() {
        let mut d = doc();
        assert!(add_inline_key(owner(&mut d), "since", Kind::Integer));
        assert!(!add_inline_key(owner(&mut d), "nested", Kind::Table));
        assert!(!add_inline_key(owner(&mut d), "name", Kind::Text));

        assert!(!Kind::VALUES.contains(&Kind::Table));

        let written = d.to_string();
        assert!(written.contains("since = 0"), "{written}");
        written.parse::<DocumentMut>().expect("still parses");
    }
}

#[cfg(test)]
mod convert_tests {
    use super::{convert_inline_key, convert_key, Kind};
    use toml_edit::DocumentMut;

    /// A table becomes an inline table in its own place, and comes back.
    /// Which layout a file uses is what the source pane exists to show, and
    /// this is the one change here that is about layout rather than type.
    #[test]
    fn a_table_and_an_inline_table_trade_places() {
        let mut d: DocumentMut = "first = 1\n\n# who\n[owner]\nname = \"D\"\n\n[last]\nz = 0\n"
            .parse()
            .expect("valid TOML");
        assert!(convert_key(d.as_table_mut(), "owner", Kind::InlineTable));
        let keys: Vec<&str> = d.as_table().iter().map(|(k, _)| k).collect();
        assert_eq!(keys, ["first", "owner", "last"]);
        assert!(d["owner"].is_inline_table(), "{d}");
        assert_eq!(
            d.to_string(),
            "first = 1\n\n# who\nowner = { name = \"D\" }\n\n[last]\nz = 0\n"
        );

        assert!(convert_key(d.as_table_mut(), "owner", Kind::Table));
        assert!(d["owner"].is_table(), "{d}");
        assert_eq!(
            d.to_string(),
            "first = 1\n\n# who\n[owner]\nname = \"D\"\n\n[last]\nz = 0\n"
        );
    }

    /// A value converts where it reads as the target and is refused where it
    /// does not, and the refusal leaves it as it was.
    #[test]
    fn a_value_converts_or_is_left_alone() {
        let mut d: DocumentMut = "n = \"44\"   # beside\n".parse().expect("valid TOML");
        assert!(convert_key(d.as_table_mut(), "n", Kind::Integer));
        assert_eq!(d.to_string(), "n = 44   # beside\n");
        assert!(!convert_key(d.as_table_mut(), "n", Kind::Boolean));
        assert_eq!(d.to_string(), "n = 44   # beside\n");
        assert!(!convert_key(d.as_table_mut(), "n", Kind::Table));
        assert!(!convert_key(d.as_table_mut(), "absent", Kind::Text));
    }

    /// Inside an inline table the same, and a table is refused outright.
    #[test]
    fn an_inline_table_converts_its_values_and_holds_no_table() {
        let mut d: DocumentMut = "t = { n = \"1\", m = { x = 1 } }\n"
            .parse()
            .expect("valid TOML");
        let inline = d["t"].as_inline_table_mut().expect("an inline table");
        assert!(convert_inline_key(inline, "n", Kind::Integer));
        assert!(!convert_inline_key(inline, "m", Kind::Table));
        assert!(!convert_inline_key(inline, "n", Kind::InlineTable));
        assert_eq!(d.to_string(), "t = { n = 1, m = { x = 1 } }\n");
    }
}

#[cfg(test)]
mod value_tests {
    use super::set_value;
    use toml_edit::{DocumentMut, Value};

    /// A changed value keeps the comment beside it and the spacing around it.
    ///
    /// In slipcase-desktop this was only ever checked through a whole save,
    /// which stays there. Here it is the operation alone: without the decor
    /// restore in `set_value`, `title = "after"` comes back with its comment
    /// gone and its alignment collapsed, and this fails on both.
    #[test]
    fn a_changed_value_keeps_its_comment_and_its_spacing() {
        let mut d: DocumentMut = "title   =   \"before\"   # beside the title\n"
            .parse()
            .expect("valid TOML");
        set_value(
            d["title"].as_value_mut().expect("a value"),
            Value::from("after"),
        );
        assert_eq!(
            d.to_string(),
            "title   =   \"after\"   # beside the title\n"
        );
    }
}
