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
