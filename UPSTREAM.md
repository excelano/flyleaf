# Upstream: what toml_edit does to a document, and what was done about it

`flyleaf-core/tests/roundtrip.rs` holds every valid TOML 1.1.0 case in
toml-test to a byte-identical round trip. Where `toml_edit` itself changes a
document, the fix belongs upstream rather than here: that is the commitment in
`PLAN.md`, and this file is the record of each finding, its reproduction,
and where it stands. Measured against `toml_edit` 0.25.13 on 2026-09-07.

Each reproduction below is the smallest that shows the behaviour, verified by
running it; the corpus cases that exercise it are named so a fix can be
checked against them and taken off `KNOWN_TO_DIFFER` in the same commit.

---

## 1. A dotted-key or header prefix segment is rendered from its first definition

**Status: filed 2026-09-07 as [toml-rs/toml#1214](https://github.com/toml-rs/toml/issues/1214).** Not found in the
tracker before filing; #163 is the nearest and is about order rather than
representation.

Every segment of a dotted key or table header except the last is rendered
from the key stored in the parent table, which is the one the first definition
wrote. A later occurrence that quotes the segment differently, or puts
whitespace before its dot, comes back as the first occurrence wrote it.

```toml
a.b = 1
"a".c = 2
```

renders as

```toml
a.b = 1
a.c = 2
```

The same for whitespace, `a . c = 2` after `a.b = 1` coming back `a. c = 2`
(the space after the dot survives, the one before it does not), and for
headers, `[a.b]` after `['a']` coming back `['a'.b]`: the `a` in the second
header is rendered as the first header spelled it.

All three are valid TOML 1.0 and 1.1, and `toml_edit` reads them correctly;
what changes is the text. This is presumably the same storage decision as
#163, a dotted key being a nested table rather than a key path, so the
prefix segments have one representation per table rather than one per
occurrence. It is reported separately because a fix for order might not fix
representation and a caller can tell them apart.

Corpus cases: `valid/key/dotted-01.toml`, `valid/key/dotted-02.toml`,
`valid/spec-1.1.0/common-7.toml`, `valid/table/empty-name.toml`,
`valid/table/with-literal-string.toml`, `valid/table/with-single-quotes.toml`.

---

## 2. Dotted keys that interleave two implicit tables are regrouped by table

**Status: known upstream, [toml-rs/toml#163](https://github.com/toml-rs/toml/issues/163),
open since 2021-09-03 and named in `toml_edit`'s own Limitations list.**
Nothing to file. The maintainer's position in the thread is that the logical
structure is what the library edits and a concrete syntax tree is a different
project, which `toml_parser` now makes possible for somebody else.

```toml
apple.type = "fruit"
orange.type = "fruit"
apple.skin = "thin"
```

renders with the two `apple` keys together and `orange` after them.

What this repository does: nothing but hold the case. A document written
this way is rare enough, and the spec itself calls the style discouraged,
that carrying a correction outside the parser is not worth what it would
cost in complexity. Corpus case: `valid/spec-1.1.0/common-9.toml`.

---

## 3. A CRLF document prints back with LF outside its multi-line strings

**Status: known upstream, [toml-rs/toml#1213](https://github.com/toml-rs/toml/issues/1213),
filed 2026-09-04 by DRMacIver.** `flyleaf_core::Document` records the file's line
ending at parse and, at render, turns every bare `\n` into `\r\n` while
leaving the ones already paired alone, which is what makes a CRLF document
with a multi-line string come back byte for byte. Corpus case:
`valid/string/multiline-escaped-crlf.toml`, which round-trips.

---

## 4. A leading byte order mark is dropped, and a file without a final newline gains one

**Status: handled in `Document`; not filed.** Both are facts about the file
rather than the document, and both are put back at render. The byte order
mark is recorded rather than stripped, because `toml_edit` accepts one of its
own and refuses a second, and stripping one here let a file with two through:
`invalid/encoding/bom-not-at-start-02.toml` caught that. Whether the
maintainers would consider either worth preserving inside `toml_edit` is a
question to ask when filing item 1, not an issue of its own.
