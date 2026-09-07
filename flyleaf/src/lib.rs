//! Tommy Flyleaf's editor, as a widget: a TOML document drawn as a tree into
//! an egui `Ui`, with every edit made through [`flyleaf_core`].
//
// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]

pub use flyleaf_core;

mod tree;

pub use tree::{render, Policy};
