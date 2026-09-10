//! The sample files published at excelano.com/flyleaf/samples/ open in this
//! editor and come back byte for byte.
//!
//! The defect this catches is a hosted sample that the application refuses,
//! or that it rewrites somewhere the person did not edit, found by whoever
//! downloaded it rather than by us. App Review downloads these files: the
//! Mac App Store submission of 0.2.1 came back on 2026-09-10 asking for
//! sample TOML at a permanent address, and that address is only as good as
//! what is at it.
//!
//! The whole published set is checked, not a chosen part of it: the glob is
//! what `packaging/samples/publish.sh` copies, so a sample added there is
//! covered without this file being touched.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::fs;
use std::path::{Path, PathBuf};

use flyleaf_core::Document;

/// The repository root, one level above this crate.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Every file the samples page serves: the job ticket, which is
/// `packaging/sample.toml` under another name, and `packaging/samples/`.
fn published() -> Vec<PathBuf> {
    let mut files = vec![root().join("packaging/sample.toml")];
    let dir = root().join("packaging/samples");
    let mut listed: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("packaging/samples is readable")
        .map(|e| e.expect("a readable entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    listed.sort();
    assert!(
        listed.len() >= 3,
        "only {} sample(s) in packaging/samples",
        listed.len()
    );
    files.extend(listed);
    files
}

#[test]
fn every_published_sample_opens_and_round_trips() {
    for path in published() {
        let name = path.file_name().expect("a file name").to_string_lossy();
        let bytes = fs::read(&path).unwrap_or_else(|e| panic!("{name} is not readable: {e}"));
        let doc = Document::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("{name} is published as TOML and was refused: {e}"));
        assert!(!doc.edited(), "{name} reads as edited before any edit");
        assert!(
            doc.render().as_bytes() == bytes,
            "{name} does not come back byte for byte"
        );
    }
}
