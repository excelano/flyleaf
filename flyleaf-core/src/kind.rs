//! What a value is, what a new one starts as, and what one can become.
//!
//! TOML has eleven kinds once the four datetime shapes are told apart, and
//! an editor that promises type-aware editing has to name all of them: a
//! local date is not an offset date-time with the time left off, and a
//! person who typed one meant it.

use toml_edit::{Array, Datetime, InlineTable, Item, Table, Value};

/// A kind of value, or a table.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A string.
    Text,
    /// An integer.
    Integer,
    /// A float.
    Float,
    /// A boolean.
    Boolean,
    /// A date and time with an offset from UTC.
    OffsetDateTime,
    /// A date and time with no offset.
    LocalDateTime,
    /// A date alone.
    LocalDate,
    /// A time alone.
    LocalTime,
    /// An array of values.
    Array,
    /// A table written on one line, `{ a = 1 }`.
    InlineTable,
    /// A table with a header, `[name]`.
    Table,
}

impl Kind {
    /// The kinds that are a single value.
    pub const SCALARS: [Self; 8] = [
        Self::Text,
        Self::Integer,
        Self::Float,
        Self::Boolean,
        Self::OffsetDateTime,
        Self::LocalDateTime,
        Self::LocalDate,
        Self::LocalTime,
    ];

    /// The kinds an inline table or an array can hold: every value.
    pub const VALUES: [Self; 10] = [
        Self::Text,
        Self::Integer,
        Self::Float,
        Self::Boolean,
        Self::OffsetDateTime,
        Self::LocalDateTime,
        Self::LocalDate,
        Self::LocalTime,
        Self::Array,
        Self::InlineTable,
    ];

    /// Every kind, in the order a picker offers them.
    pub const ALL: [Self; 11] = [
        Self::Text,
        Self::Integer,
        Self::Float,
        Self::Boolean,
        Self::OffsetDateTime,
        Self::LocalDateTime,
        Self::LocalDate,
        Self::LocalTime,
        Self::Array,
        Self::InlineTable,
        Self::Table,
    ];

    /// What it is called where somebody chooses it, or reads it beside a row.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Boolean => "boolean",
            Self::OffsetDateTime => "offset date-time",
            Self::LocalDateTime => "local date-time",
            Self::LocalDate => "local date",
            Self::LocalTime => "local time",
            Self::Array => "array",
            Self::InlineTable => "inline table",
            Self::Table => "table",
        }
    }

    /// The kind of a value.
    #[must_use]
    pub fn of_value(v: &Value) -> Self {
        match v {
            Value::String(_) => Self::Text,
            Value::Integer(_) => Self::Integer,
            Value::Float(_) => Self::Float,
            Value::Boolean(_) => Self::Boolean,
            Value::Datetime(d) => Self::of_datetime(d.value()),
            Value::Array(_) => Self::Array,
            Value::InlineTable(_) => Self::InlineTable,
        }
    }

    /// The kind of an item; `None` for a removed one and for an array of
    /// tables, which is not a value and not one table.
    #[must_use]
    pub fn of_item(item: &Item) -> Option<Self> {
        match item {
            Item::Value(v) => Some(Self::of_value(v)),
            Item::Table(_) => Some(Self::Table),
            Item::None | Item::ArrayOfTables(_) => None,
        }
    }

    fn of_datetime(d: &Datetime) -> Self {
        match (d.date.is_some(), d.time.is_some(), d.offset.is_some()) {
            (true, true, true) => Self::OffsetDateTime,
            (true, true, false) => Self::LocalDateTime,
            (true, false, _) => Self::LocalDate,
            (false, _, _) => Self::LocalTime,
        }
    }

    /// What a new one starts as, where only a value will do.
    #[must_use]
    pub fn as_value(self) -> Option<Value> {
        match self.item() {
            Item::Value(v) => Some(v),
            _ => None,
        }
    }

    /// What a new one starts as: empty, zero, false, or the epoch, which is a
    /// date somebody will replace rather than a guess at the one they meant.
    ///
    /// # Panics
    ///
    /// Never: the four datetime literals are fixed and parse, and a test
    /// makes every kind once.
    #[must_use]
    pub fn item(self) -> Item {
        let datetime = |text: &str| {
            Item::Value(Value::from(
                text.parse::<Datetime>().expect("a fixed datetime literal"),
            ))
        };
        match self {
            Self::Text => Item::Value(Value::from("")),
            Self::Integer => Item::Value(Value::from(0_i64)),
            Self::Float => Item::Value(Value::from(0.0_f64)),
            Self::Boolean => Item::Value(Value::from(false)),
            Self::OffsetDateTime => datetime("1970-01-01T00:00:00Z"),
            Self::LocalDateTime => datetime("1970-01-01T00:00:00"),
            Self::LocalDate => datetime("1970-01-01"),
            Self::LocalTime => datetime("00:00:00"),
            Self::Array => Item::Value(Value::Array(Array::new())),
            Self::InlineTable => Item::Value(Value::InlineTable(InlineTable::new())),
            Self::Table => Item::Table(Table::new()),
        }
    }
}

/// The value as another kind, where it reads as one; `None` where the change
/// would be a guess.
///
/// What converts: a scalar to text, always, as the text it is written with;
/// text to anything it parses as; an integer to a float and a whole float
/// back; a datetime to another shape by dropping what the target lacks, or
/// filling what it lacks with midnight, the epoch, or UTC; any value to a
/// one-element array, and a one-element array back to its element. What does
/// not: a boolean to a number and back, since `true` is not `1` in TOML; a
/// float with a fraction to an integer; anything to an inline table. The
/// returned value carries no decor; [`crate::set_value`] keeps the old one's.
#[must_use]
pub fn convert(v: &Value, to: Kind) -> Option<Value> {
    if Kind::of_value(v) == to {
        return Some(v.clone());
    }
    match to {
        Kind::Text => Some(Value::from(scalar_text(v)?)),
        Kind::Integer => match v {
            Value::String(s) => s.value().trim().parse::<i64>().ok().map(Value::from),
            Value::Float(f) if f.value().fract() == 0.0 && f.value().is_finite() => {
                // Whole, finite, and within range: a float past i64 saturates
                // silently on `as`, so the range is checked first, and what
                // the lint calls truncation cannot happen to a whole number.
                let x = *f.value();
                #[allow(clippy::cast_possible_truncation)]
                (x.abs() < 9.2e18).then(|| Value::from(x as i64))
            }
            Value::Array(a) => single(a).and_then(|e| convert(e, to)),
            _ => None,
        },
        Kind::Float => match v {
            Value::String(s) => s.value().trim().parse::<f64>().ok().map(Value::from),
            // Precision is lost past 2^53, which is the conversion asked for
            // and what `as` does.
            #[allow(clippy::cast_precision_loss)]
            Value::Integer(i) => Some(Value::from(*i.value() as f64)),
            Value::Array(a) => single(a).and_then(|e| convert(e, to)),
            _ => None,
        },
        Kind::Boolean => match v {
            Value::String(s) => match s.value().trim() {
                "true" => Some(Value::from(true)),
                "false" => Some(Value::from(false)),
                _ => None,
            },
            Value::Array(a) => single(a).and_then(|e| convert(e, to)),
            _ => None,
        },
        Kind::OffsetDateTime | Kind::LocalDateTime | Kind::LocalDate | Kind::LocalTime => {
            let d = match v {
                Value::String(s) => s.value().trim().parse::<Datetime>().ok()?,
                Value::Datetime(d) => *d.value(),
                Value::Array(a) => return single(a).and_then(|e| convert(e, to)),
                _ => return None,
            };
            Some(Value::from(reshape(d, to)))
        }
        Kind::Array => {
            // The value's own decor is the space before it on its line, which
            // inside brackets would read `[ 44]`.
            let mut element = v.clone();
            *element.decor_mut() = toml_edit::Decor::default();
            let mut a = Array::new();
            a.push_formatted(element);
            Some(Value::Array(a))
        }
        Kind::InlineTable | Kind::Table => None,
    }
}

/// A scalar as text: the string itself, or the text a number, boolean or
/// datetime is written with.
fn scalar_text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.value().clone()),
        Value::Integer(i) => Some(i.value().to_string()),
        Value::Float(f) => Some(f.value().to_string()),
        Value::Boolean(b) => Some(b.value().to_string()),
        Value::Datetime(d) => Some(d.value().to_string()),
        Value::Array(a) => single(a).and_then(scalar_text),
        Value::InlineTable(_) => None,
    }
}

/// The one element of a one-element array.
fn single(a: &Array) -> Option<&Value> {
    (a.len() == 1).then(|| a.get(0)).flatten()
}

/// A datetime as another shape: what the target lacks is dropped, and what
/// it has that the source lacks is filled with the epoch's date, midnight, or
/// UTC.
fn reshape(d: Datetime, to: Kind) -> Datetime {
    let epoch_date = "1970-01-01".parse::<Datetime>().expect("a date").date;
    let midnight = "00:00:00".parse::<Datetime>().expect("a time").time;
    let utc = "1970-01-01T00:00:00Z"
        .parse::<Datetime>()
        .expect("an offset")
        .offset;
    let date = d.date.or(epoch_date);
    let time = d.time.or(midnight);
    match to {
        Kind::OffsetDateTime => Datetime {
            date,
            time,
            offset: d.offset.or(utc),
        },
        Kind::LocalDateTime => Datetime {
            date,
            time,
            offset: None,
        },
        Kind::LocalDate => Datetime {
            date,
            time: None,
            offset: None,
        },
        _ => Datetime {
            date: None,
            time,
            offset: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{convert, Kind};
    use toml_edit::{DocumentMut, Value};

    fn value(text: &str) -> Value {
        let doc: DocumentMut = format!("v = {text}\n").parse().expect("valid TOML");
        doc["v"].as_value().expect("a value").clone()
    }

    fn text_of(v: &Value) -> String {
        v.to_string().trim().to_owned()
    }

    /// Every datetime shape is told apart, because a local date is not an
    /// offset date-time with the time left off.
    #[test]
    fn the_four_datetime_shapes_are_told_apart() {
        assert_eq!(
            Kind::of_value(&value("1979-05-27T07:32:00Z")),
            Kind::OffsetDateTime
        );
        assert_eq!(
            Kind::of_value(&value("1979-05-27T07:32:00")),
            Kind::LocalDateTime
        );
        assert_eq!(Kind::of_value(&value("1979-05-27")), Kind::LocalDate);
        assert_eq!(Kind::of_value(&value("07:32:00")), Kind::LocalTime);
    }

    /// What a new value of each kind starts as is that kind, and is valid
    /// TOML: a picker that offered a kind whose starting value was another
    /// would lie.
    #[test]
    fn every_kind_starts_as_itself() {
        for kind in Kind::ALL {
            let item = kind.item();
            assert_eq!(Kind::of_item(&item), Some(kind), "{kind:?}");
            let mut doc = DocumentMut::new();
            doc.as_table_mut().insert("k", item);
            doc.to_string().parse::<DocumentMut>().expect("valid TOML");
        }
    }

    /// The conversions that read as the target, and what they produce.
    #[test]
    fn conversions_that_read_as_the_target_succeed() {
        let cases: &[(&str, Kind, &str)] = &[
            ("44", Kind::Text, "\"44\""),
            ("1.5", Kind::Text, "\"1.5\""),
            ("true", Kind::Text, "\"true\""),
            ("1979-05-27", Kind::Text, "\"1979-05-27\""),
            ("\"44\"", Kind::Integer, "44"),
            ("\" 44 \"", Kind::Integer, "44"),
            ("2.0", Kind::Integer, "2"),
            ("\"1.5\"", Kind::Float, "1.5"),
            ("2", Kind::Float, "2.0"),
            ("\"true\"", Kind::Boolean, "true"),
            ("\"1979-05-27\"", Kind::LocalDate, "1979-05-27"),
            (
                "1979-05-27T07:32:00Z",
                Kind::LocalDateTime,
                "1979-05-27T07:32:00",
            ),
            ("1979-05-27T07:32:00Z", Kind::LocalDate, "1979-05-27"),
            ("1979-05-27T07:32:00Z", Kind::LocalTime, "07:32:00"),
            (
                "1979-05-27T07:32:00",
                Kind::OffsetDateTime,
                "1979-05-27T07:32:00Z",
            ),
            ("1979-05-27", Kind::LocalDateTime, "1979-05-27T00:00:00"),
            ("07:32:00", Kind::LocalDateTime, "1970-01-01T07:32:00"),
            ("44", Kind::Array, "[44]"),
            ("[44]", Kind::Integer, "44"),
            ("[\"x\"]", Kind::Text, "\"x\""),
            ("44", Kind::Integer, "44"),
        ];
        for (from, to, expected) in cases {
            let got = convert(&value(from), *to)
                .unwrap_or_else(|| panic!("{from} -> {to:?} was refused"));
            assert_eq!(text_of(&got), *expected, "{from} -> {to:?}");
            assert_eq!(
                Kind::of_value(&got),
                *to,
                "{from} -> {to:?} is not a {to:?}"
            );
        }
    }

    /// The conversions that would be a guess are refused rather than guessed.
    #[test]
    fn conversions_that_would_guess_are_refused() {
        let cases: &[(&str, Kind)] = &[
            ("true", Kind::Integer),
            ("1", Kind::Boolean),
            ("1.5", Kind::Integer),
            ("\"forty-four\"", Kind::Integer),
            ("\"yes\"", Kind::Boolean),
            ("\"tomorrow\"", Kind::LocalDate),
            ("1979-05-27", Kind::Integer),
            ("[1, 2]", Kind::Integer),
            ("44", Kind::InlineTable),
            ("{ a = 1 }", Kind::Text),
            ("inf", Kind::Integer),
            ("1e300", Kind::Integer),
        ];
        for (from, to) in cases {
            assert!(
                convert(&value(from), *to).is_none(),
                "{from} -> {to:?} was allowed"
            );
        }
    }
}
