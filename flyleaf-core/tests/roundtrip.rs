//! Every valid TOML 1.1.0 case in toml-test comes back byte for byte after a
//! parse and a render, and every invalid one is refused.
//!
//! Round-trip fidelity is the product, and this is the measurement of it
//! against the largest neutral corpus there is. The cases live in
//! `tests/toml-test/`, with their license and the commit they came from.
//!
//! Where `toml_edit` itself changes a valid document, the case is named below
//! with the reason, and the test asserts that it still differs: a case that
//! starts round-tripping after a `toml_edit` bump is a fix to notice and take
//! off the list, not a pass to be quietly absorbed.

use std::fs;
use std::path::{Path, PathBuf};

use flyleaf_core::Document;

/// Valid cases `toml_edit` 0.25.13 does not reproduce, and why. Measured on
/// 2026-09-07; every one is upstream's rather than this crate's.
///
/// Two shapes. **A non-leaf dotted-key segment, or a table-header segment,
/// is rendered from the key's first definition rather than from each
/// occurrence**: `"count".c` after `count.a` comes back `count.c`, `[a.'b']`
/// after `['a']` comes back `['a'.'b']`, and the whitespace before a dot goes
/// with it. **Dotted keys that interleave two implicit tables are regrouped
/// by table**: `apple.type`, `orange.type`, `apple.skin` come back with the
/// apples together. Neither is a TOML 1.1 matter, and neither can be put back
/// from outside the parser, which is why they are here and not in
/// `Document`.
const KNOWN_TO_DIFFER: &[(&str, &str)] = &[
    (
        "valid/key/dotted-01.toml",
        "quoted non-leaf segment rendered bare",
    ),
    (
        "valid/key/dotted-02.toml",
        "quoting and whitespace of non-leaf segments",
    ),
    (
        "valid/spec-1.1.0/common-7.toml",
        "whitespace before a dot in a non-leaf segment",
    ),
    (
        "valid/spec-1.1.0/common-9.toml",
        "interleaved dotted keys regrouped by table",
    ),
    (
        "valid/table/empty-name.toml",
        "\"\" as a header segment rendered as ''",
    ),
    (
        "valid/table/with-literal-string.toml",
        "header segment re-quoted from its first definition",
    ),
    (
        "valid/table/with-single-quotes.toml",
        "header segment re-quoted from its first definition",
    ),
];

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/toml-test")
}

/// The cases on the 1.1.0 list, as `valid/...` or `invalid/...` paths.
fn cases(prefix: &str) -> Vec<String> {
    let list = fs::read_to_string(corpus().join("files-toml-1.1.0")).expect("the case list");
    let mut names: Vec<String> = list
        .lines()
        .filter(|l| l.starts_with(prefix))
        .map(str::to_owned)
        .collect();
    names.sort();
    assert!(names.len() > 100, "only {} {prefix} cases", names.len());
    names
}

/// A valid document renders as the bytes it was read from, byte order mark,
/// line ending and final newline included, except where `toml_edit` itself
/// changes it, and every one of those is named.
#[test]
fn every_valid_case_round_trips_or_is_known_not_to() {
    let mut unexpected = Vec::new();
    let mut fixed = Vec::new();
    for name in cases("valid/") {
        let bytes = fs::read(corpus().join(&name)).expect("a readable case");
        let doc = Document::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("{name} is valid TOML and was refused: {e}"));
        assert!(!doc.edited(), "{name} reads as edited before any edit");
        let identical = doc.render().as_bytes() == bytes;
        let known = KNOWN_TO_DIFFER.iter().any(|(n, _)| *n == name);
        match (identical, known) {
            (true, false) | (false, true) => {}
            (false, false) => unexpected.push(name),
            (true, true) => fixed.push(name),
        }
    }
    assert!(
        unexpected.is_empty(),
        "these valid cases do not round-trip and are not on the list:\n  {}",
        unexpected.join("\n  ")
    );
    assert!(
        fixed.is_empty(),
        "these cases now round-trip; take them off KNOWN_TO_DIFFER and say so in the commit:\n  {}",
        fixed.join("\n  ")
    );
}

/// Every invalid case is refused. A parser that accepts one would let this
/// editor open a file it cannot promise to write back correctly.
#[test]
fn every_invalid_case_is_refused() {
    let accepted: Vec<String> = cases("invalid/")
        .into_iter()
        .filter(|name| {
            let bytes = fs::read(corpus().join(name)).expect("a readable case");
            Document::from_bytes(&bytes).is_ok()
        })
        .collect();
    assert!(
        accepted.is_empty(),
        "these invalid cases were accepted:\n  {}",
        accepted.join("\n  ")
    );
}

/// The list of known differences names real cases, so a renamed or removed
/// case cannot leave a stale entry that guards nothing.
#[test]
fn every_known_difference_names_a_case() {
    for (name, _) in KNOWN_TO_DIFFER {
        assert!(corpus().join(name).is_file(), "{name} is not in the corpus");
    }
}
