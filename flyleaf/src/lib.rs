//! Tommy Flyleaf's editor, as a widget: a TOML document drawn as a tree into
//! an egui `Ui`, with every edit made through [`flyleaf_core`].
//
// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]

pub use flyleaf_core;

mod source;
mod tree;

pub use source::source;
pub use tree::{forget_typing, open_all, render, Policy};

/// The messages this crate draws, in whatever language it has been told.
///
/// `potext::catalog!` declares the catalogue **in this crate**, which is the
/// point of it being a macro rather than a global inside `potext`: the tree is
/// drawn inside somebody else's window — slipcase-desktop's — and a widget has
/// to carry its own strings rather than borrow the application's.
mod i18n {
    potext::catalog!();
}

/// Every language this crate's own strings are translated into.
///
/// The application binary and a host application both go through
/// [`set_language`]; neither is handed the catalogues, because the only thing
/// that should cross a crate boundary is a language tag. A catalogue passed
/// across would be a type, and two versions of this crate in one graph would
/// make it two.
const CATALOGUES: &[(&str, &str)] = &[
    ("de", include_str!("../../po/de.po")),
    // Debug builds alone, so a release of this crate carries nothing of it.
    // A host running its own pseudolocale asks for this tag and gets a tree
    // that answers in the same alphabet as the window around it.
    #[cfg(debug_assertions)]
    ("en-x-pseudo", include_str!("../../po/en-x-pseudo.po")),
];

/// Draw this crate's strings in the named language, where there is one for it.
///
/// The tag is a locale name in any of the spellings the platforms use —
/// `de`, `de-AT`, `de_DE.UTF-8` — and the closest catalogue is chosen: the
/// exact tag first, then the language alone. Returns the tag chosen, or `None`
/// where nothing matched and the tree stays in the English its call sites are
/// written in.
///
/// **A host application calls this rather than letting the widget ask.** An
/// application has already decided what language it is in, by whatever rule it
/// keeps, and a tree in a different language from the window around it would be
/// worse than an English one. `potext::preferred()` is that rule for anybody
/// who wants the platform's answer to pass on.
///
/// Calling it twice does nothing the second time: a language change is a
/// restart, which is what it is everywhere else.
#[must_use]
pub fn set_language(tag: &str) -> Option<String> {
    i18n::set_language(tag, CATALOGUES)
}
