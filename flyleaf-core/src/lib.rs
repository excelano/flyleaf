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

use toml_edit::{Datetime, InlineTable, Item, Key, Table, Value};

/// What a new key starts as.
///
/// The scalar types, and a table to put them in. An array and an array of
/// tables are structure inside structure and are not offered yet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NewKey {
    /// An empty string.
    Text,
    /// Zero.
    Integer,
    /// Zero.
    Float,
    /// False.
    Boolean,
    /// The epoch, which is a date somebody will replace rather than a guess at
    /// the one they meant.
    Datetime,
    /// An empty table to put keys in.
    Table,
}

impl NewKey {
    /// The kinds an inline table can hold.
    ///
    /// It holds values, and a table is not one. A table inside an inline table
    /// would have to be another inline table, which is structure inside
    /// structure and is not offered any more than an array is.
    pub const SCALARS: [Self; 5] = [
        Self::Text,
        Self::Integer,
        Self::Float,
        Self::Boolean,
        Self::Datetime,
    ];

    /// Every kind, in the order a picker offers them.
    pub const ALL: [Self; 6] = [
        Self::Text,
        Self::Integer,
        Self::Float,
        Self::Boolean,
        Self::Datetime,
        Self::Table,
    ];

    /// What it is called where somebody chooses it.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Boolean => "boolean",
            Self::Datetime => "date and time",
            Self::Table => "table",
        }
    }

    /// What it starts as, where only a value will do.
    fn as_value(self) -> Option<Value> {
        match self.item() {
            Item::Value(v) => Some(v),
            _ => None,
        }
    }

    fn item(self) -> Item {
        match self {
            Self::Text => Item::Value(Value::from("")),
            Self::Integer => Item::Value(Value::from(0_i64)),
            Self::Float => Item::Value(Value::from(0.0_f64)),
            Self::Boolean => Item::Value(Value::from(false)),
            Self::Datetime => Item::Value(Value::from(
                "1970-01-01T00:00:00Z"
                    .parse::<Datetime>()
                    .expect("the epoch is a datetime"),
            )),
            Self::Table => Item::Table(Table::new()),
        }
    }
}

/// Add a key to a table.
///
/// Refuses an empty name and one already there: inserting over an existing key
/// would replace it, and losing a value to a name collision is not what
/// pressing Add asks for.
pub fn add_key(t: &mut Table, name: &str, kind: NewKey) -> bool {
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
pub fn add_inline_key(t: &mut InlineTable, name: &str, kind: NewKey) -> bool {
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
    use super::{add_key, remove_key, rename_key, NewKey};
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
        assert!(add_key(d.as_table_mut(), "author", NewKey::Text));
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
        assert!(!add_key(d.as_table_mut(), "first", NewKey::Integer));
        assert!(!add_key(d.as_table_mut(), "", NewKey::Integer));
        assert!(d.to_string().contains("first = \"one\""));
    }

    /// Every kind a picker offers produces a document that still parses.
    #[test]
    fn every_kind_of_new_key_is_valid_toml() {
        for kind in NewKey::ALL {
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
    use super::{add_inline_key, remove_inline_key, rename_inline_key, NewKey};
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
        assert!(add_inline_key(owner(&mut d), "since", NewKey::Integer));
        assert!(!add_inline_key(owner(&mut d), "nested", NewKey::Table));
        assert!(!add_inline_key(owner(&mut d), "name", NewKey::Text));

        assert!(!NewKey::SCALARS.contains(&NewKey::Table));

        let written = d.to_string();
        assert!(written.contains("since = 0"), "{written}");
        written.parse::<DocumentMut>().expect("still parses");
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
