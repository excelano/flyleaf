//! The operations on an array: an element added at the end in the array's
//! own style, one removed, one changed.

use toml_edit::{Array, Value};

use crate::{convert, set_value, Kind};

/// Add an element of a kind at the end, in the style of the element before
/// it.
///
/// `toml_edit` writes a pushed element with a default decor, which turns a
/// multi-line array's last line into `  2,\n3]`. The new element takes the
/// decor of the one before it instead, so a multi-line array stays one and a
/// one-line array stays one. Refuses a table, which is not a value.
pub fn push_element(a: &mut Array, kind: Kind) -> bool {
    let Some(mut value) = kind.as_value() else {
        return false;
    };
    if let Some(last) = a.iter().last() {
        *value.decor_mut() = last.decor().clone();
    }
    a.push_formatted(value);
    true
}

/// Remove the element at an index.
pub fn remove_element(a: &mut Array, index: usize) -> bool {
    if index >= a.len() {
        return false;
    }
    a.remove(index);
    true
}

/// Change the element at an index, keeping the decor it was written with.
pub fn set_element(a: &mut Array, index: usize, new: Value) -> bool {
    match a.get_mut(index) {
        Some(slot) => {
            set_value(slot, new);
            true
        }
        None => false,
    }
}

/// Change the element at an index to another kind, where it reads as one.
pub fn convert_element(a: &mut Array, index: usize, to: Kind) -> bool {
    let Some(slot) = a.get_mut(index) else {
        return false;
    };
    match convert(slot, to) {
        Some(new) => {
            set_value(slot, new);
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{convert_element, push_element, remove_element, set_element};
    use crate::Kind;
    use toml_edit::{DocumentMut, Value};

    fn doc(text: &str) -> DocumentMut {
        text.parse().expect("valid TOML")
    }

    fn array(d: &mut DocumentMut) -> &mut toml_edit::Array {
        d["a"].as_array_mut().expect("an array")
    }

    /// An element added to a one-line array lands on that line, and one
    /// added to a multi-line array lands on a line of its own with the same
    /// indentation. Without copying the decor, the second came out as
    /// `  2,\n3]`, which is what this fails on.
    #[test]
    fn an_added_element_takes_the_style_of_the_one_before() {
        let mut d = doc("a = [1, 2]\n");
        assert!(push_element(array(&mut d), Kind::Integer));
        assert_eq!(d.to_string(), "a = [1, 2, 0]\n");

        let mut d = doc("a = [\n  1,\n  2,\n]\n");
        assert!(push_element(array(&mut d), Kind::Integer));
        assert_eq!(d.to_string(), "a = [\n  1,\n  2,\n  0,\n]\n");

        let mut d = doc("a = []\n");
        assert!(push_element(array(&mut d), Kind::Text));
        assert_eq!(d.to_string(), "a = [\"\"]\n");
    }

    /// A table is not a value and cannot go in an array.
    #[test]
    fn a_table_is_refused() {
        let mut d = doc("a = [1]\n");
        assert!(!push_element(array(&mut d), Kind::Table));
        assert_eq!(d.to_string(), "a = [1]\n");
    }

    /// Removing an element leaves the others as they were written, and an
    /// index past the end is refused rather than panicking.
    #[test]
    fn an_element_is_removed_in_place() {
        let mut d = doc("a = [1, 2, 3]   # three\n");
        assert!(remove_element(array(&mut d), 1));
        assert_eq!(d.to_string(), "a = [1, 3]   # three\n");
        assert!(!remove_element(array(&mut d), 5));
    }

    /// A changed element keeps its spacing, and a converted one is the new
    /// kind in the old place.
    #[test]
    fn an_element_is_changed_or_converted_where_it_is() {
        let mut d = doc("a = [ 1 , 2 ]\n");
        assert!(set_element(array(&mut d), 0, Value::from(9)));
        assert_eq!(d.to_string(), "a = [ 9 , 2 ]\n");
        assert!(convert_element(array(&mut d), 1, Kind::Text));
        assert_eq!(d.to_string(), "a = [ 9 , \"2\" ]\n");
        assert!(!convert_element(array(&mut d), 1, Kind::Boolean));
        assert!(!set_element(array(&mut d), 7, Value::from(0)));
    }
}
