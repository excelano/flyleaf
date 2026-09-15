# Tommy Flyleaf — Design Document

What this application is and what it commits to. It is the authority on the
application: where building contradicts it, change it here in the same change.

---

## 1. What this is

A standalone, structure-aware TOML editor. Desktop GUI, cross-platform, Rust
with egui/eframe, and the same editor slipcase-desktop draws inside its own
window.

The gap it fills: TOML tooling is editor plugins and LSPs (Taplo, Tombi),
path-based CLIs (tomlpipe, dasel, tomlq), or config editors tied to one
application (mise). There is no general-purpose interactive editor that treats
the document as a tree, edits values by their type, and keeps comments and
formatting through a save.

The name reads as a person's — a bookbinder's apprentice. The flyleaf is the
blank page inside a book's cover, the one place in a bound book meant for
somebody to write on later: write on the page provided and leave the rest as it
was bound, which is what the editor does to a file.

## 2. Two crates, and what the split buys

`flyleaf-core` is the document model, the edit operations, undo, and what a save
writes. No user interface. `flyleaf` is the editor as an egui widget, which is
what slipcase-desktop depends on, plus the application binary, which is a thin
shell around it. The binary's own dependencies sit behind a default `app`
feature, so a consumer of the widget carries no window framework it did not ask
for.

**The rule that makes the split worth having: the widget never calls a
`toml_edit` mutation directly.** Every change goes through core, so undo has one
chokepoint rather than one per call site.

**One `toml_edit` across the fleet.** `flyleaf-core`, `slpc` and
slipcase-desktop must resolve to a single version, because slipcase-desktop
holds a `DocumentMut` from one and hands it to the other; two versions in the
graph make that two types and it stops compiling. Both manifests say so, and
they move together.

**One editor, not two that drift.** The widget lives here and slipcase-desktop
depends on it. Where the editor needs behaviour it lacks, the behaviour goes
here and both applications get it.

## 3. What it commits to

**Round-trip fidelity.** Comments, key order, whitespace, quoting style and
table layout survive edits made elsewhere in the document. `toml_edit`, never
`toml`.

**TOML 1.1.0**, because the Slipcase specification requires it and the editor is
shared. `toml_edit` carries `+spec-1.1.0` and round-trips every 1.1 feature
byte-identically. It refuses non-ASCII bare keys, which is correct; 1.1.0 final
kept bare keys ASCII. Where the corpus finds a real gap the path is upstream,
not a fork and not a downgrade.

**Structure-aware, not a text editor.** The tree is the primary view.

**One GUI binary with no CLI subcommands.** On Windows an executable is either
GUI-subsystem or console-subsystem, and one that tries to be both ends up in
`AttachConsole` hacks; a store build is not on `PATH` anyway. A command-line
tool over the same engine would be a separate binary in this workspace over
`flyleaf-core` — deferred, not rejected, and §11 says what it would have to be.

**Pure Rust, cross-platform, WASM-capable.**

## 4. The tree

Every row of the document is drawn: tables, arrays of tables, inline tables,
key/value pairs, with expand and collapse, and selection as the row holding
keyboard focus, returned from `render` as its path.

**Collapsed by default above two hundred entries**, because a thousand open rows
cannot be read; memory is tolerable either way, at about 10 KB a row.
`flyleaf::open_all` opens or closes everything on request. Virtualisation stays
deferred until a measurement says collapsing is not enough.

**The widget knows nothing about Slipcase.** Which keys are protected, and how a
protected string is displayed, come in as a policy the host supplies.

## 5. Editing: kinds, conversion, undo

`flyleaf_core::Kind` names all eleven kinds with the four datetime shapes told
apart. Every row carries a button reading its kind, whose menu offers the others
the container can hold, enabled only where `flyleaf_core::convert` says the
value reads as the target: a scalar to text always, text to whatever it parses
as, an integer to a float and a whole float back, a datetime to another shape by
dropping or filling with the epoch, midnight or UTC, any value to a one-element
array and back. A boolean does not become a number, a float with a fraction does
not become an integer, and nothing becomes an inline table; those are shown
disabled rather than refused after the press. A table becomes an inline table
and back in its place with its comment carried across, which is the one
conversion about layout rather than type.

**Undo is a copy of the rendered text per step**, held in `flyleaf_core::Document`,
with steps naming the same row coalesced so a word typed into a field undoes as
a word. Undo parses the previous text back, which loses nothing. The shell takes
Ctrl+Z and Ctrl+Shift+Z before the tree draws, so a focused field cannot answer
with its own undo, and calls `flyleaf::forget_typing` first, because a key field
commits its buffer on blur and would otherwise rename the row back.

**A float is drawn as text through `buffered_text`**, the way a datetime is,
parsed when it is committed and left alone otherwise, so the writer's spelling
of it survives until it is edited. `egui::DragValue` cannot draw one: it clamps
what it is handed, which turns a `nan` into an `inf` and reports the clamp as an
edit, and it rounds what it shows, which an editor claiming fidelity cannot do.

**A field commits what was typed, not what it shows**, because egui takes focus
away on Escape at the start of a frame and a field left that way would reseed
its text before it could compare.

**Comments are edited in every place TOML allows one.** Core reads and writes
each slot on `toml_edit`'s decor, keeping the blank lines around a comment block
and the indentation of the key below it, since one decor holds all three. The
one thing not kept is a blank line between two comment lines of a single block.

## 6. The source pane

A right-hand panel, on by default, drawing `Document::render` one monospace line
per row inside a scroll area that lays out only the visible rows, so a
five-thousand-line lockfile costs what fits on screen.

It exists because a general editor's users hand-edit `Cargo.toml` and think of
it as a file, and the tree hides exactly what they care about: inline table
against `[header]`, quoting style, where the comments went. Half its value is
answering *what will a save write*.

**The sync goes through `Document::lines_of`.** `DocumentMut` keeps no positions
once it can be edited, so the rendered text is parsed again as
`toml_edit::Document`, which does, and the selected path's span becomes a line
range. That parse runs only when the selection or the document changes. The row
with focus is highlighted and scrolled a third of the way down the frame the
selection changes, and left alone after.

## 7. Files, saving, and the sandbox

`Document::from_path` and `Document::save_to` live in core behind a default `fs`
feature. **A save is written to a sibling and renamed over the file**, so a
failure anywhere leaves the original whole.

The dialog runs on its own thread and is polled once a frame, since a blocking
dialog freezes the window. A close or an open with unsaved changes is refused
for the frame and asked about, with Save, Don't save and Cancel; a save that
fails keeps the prompt up with the reason in it.

**The sibling rename is what the macOS App Sandbox refuses.** A grant covers the
file a person chose and not the directory holding it, so the rewrite has to wait
in `NSItemReplacementDirectory`, asked for with the target's URL so it lands on
the target's volume, and land through `replaceItemAtURL:`. Under `TMPDIR` it
fails with `EXDEV` for any file off the boot volume. That arm is in core's
`save_to`.

**A round-trip corpus holds all of this.** The TOML 1.1.0 cases of `toml-test`
are vendored under `flyleaf-core/tests/toml-test/` — 220 valid, 494 invalid —
and every valid one must render byte-identically through `Document` while every
invalid one must be refused. Three things `toml_edit` drops and `Document`
records and reapplies: a leading byte order mark, CRLF line endings, and a
missing final newline. Seven cases remain that cannot be repaired from outside
the parser, each named in the test with its reason, and the test asserts they
*still* differ so that an upstream fix is noticed rather than absorbed. Two
shapes, both upstream candidates against `toml-rs/toml`: a non-leaf dotted-key
or table-header segment rendered from the key's first definition rather than
each occurrence, losing its quoting and the whitespace before its dot; and
dotted keys interleaving two implicit tables being regrouped by table.

## 8. The web arm

The same shell with two arms, each marked with its target: on disk a path, a
thread for the dialog and the atomic write; in a browser a name, a future for
the dialog, a file read as bytes from the picker, and a save as a download of
what the document renders to when the dialog goes up. Core's `fs` feature is on
for the native build and off for the web one, because `tempfile` has no wasm
build.

Two things a browser cannot do and the shell does not pretend to: a close
request, so the unsaved prompt appears only before an open there; and waiting
for a download to land, so Save on the web starts the download and goes on.

**The bundle is built for size**: a `web` profile with `opt-level = "z"`, fat
LTO, one codegen unit and `panic = "abort"`, and eframe with `glow` rather than
its default `wgpu` renderer on the wasm target alone, which keeps wgpu-core and
naga out of the page. That is 4.8 MB, 1.9 MB gzipped. What remains is egui, its
default fonts and the editor, which is the product.

The host serves `.wasm` as `application/wasm` unasked but compresses only text
types, so a `.gz` and a `.br` are written beside the wasm and a committed
`.htaccess` serves them by rewrite: 1.5 MB over the wire with brotli, the bytes
checked against the original by hash.

## 9. Packaging and the two stores

Debian for the Excelano apt repository, MSIX for the Microsoft Store, a signed
sandboxed bundle for the Mac App Store, crates.io, and a hosted WASM demo.
`packaging/` holds one directory per platform.

**Windows.** `uninstall.ps1` deletes a `UserChoice` by name from its parent key,
which needs only DELETE and works unelevated, reads every delete back, and
refuses if a key survives: Explorer writes a *Deny SetValue* rule on that key,
so a delete that opens it for writing fails, and .NET's `DeleteSubKeyTree` reads
that failure as the key being missing and returns quietly. `check-install.ps1`
installs, plants a `UserChoice` the way Explorer writes one, deny rule and all,
uninstalls, and reads back that everything is gone. `AppxManifest.xml.in` is
XML, comments included, and `makepri` refuses the substituted manifest if it is
not. `screenshot.ps1` waits long enough for a cold start, where a 15 MB binary
the disk has never read is slower than the association is.

**macOS.** The private-symbol check reads the public headers rather than a list
of names, so that the public symbols winit links through ApplicationServices and
ColorSync are not refused. Rank Alternate is a tie-break and not a refusal. The
sandbox container is not empty and the privacy statement says so: after one open
through the panel, `~/Library/Containers/com.excelano.flyleaf` holds a
preferences plist.

**One blemish, seen in a store screenshot and not fixed:** a multi-line string
shows a missing-glyph box at each line break, because the tree edits every
string in a single-line field.

**An editor of files hands a reviewer the files.** `excelano.com/flyleaf/samples/`
serves four `.toml` files behind download links, three of them in
`packaging/samples/` and the fourth `sample.toml` under the name the page gives
it. `samples/publish.sh` copies the set into the site working copy in one
command, so the copies cannot drift; on the site side they are committed rather
than deployed, unlike the web bundle, because the address has to survive a
deploy that forgets them; and `flyleaf-core/tests/samples.rs` holds every file in
the set to a byte-identical render, because a published sample the editor
rewrites demonstrates the opposite of the product. The page is written for
anyone who wants something to open rather than for App Review alone.

**The stores and crates.io are separate channels and their numbers need not
agree.** A `cargo publish` rebuilds no submission and a submission in review
holds no crate. What version the next release carries is decided when there is a
release to cut.

**A catalogue is packaged only if it sits under the crate that reads it.**
`include_str!` reaching above a crate's own directory compiles locally and fails
in `cargo package`, which copies only what is under the crate root: the tarball
carries no catalogue and the publish dies verifying a build that cannot compile.
The check is `cargo package -p <crate> --list`.

## 10. The language a person reads

German where the desktop asks for German, English everywhere else, through
`potext` and the catalogues in `flyleaf/po/`.

**A message is looked up by its English text, never by a key**, so a call site
reads as the sentence a person sees and an untranslated message is the original
rather than a placeholder. **A translation that has gone stale is not shown:**
`msgmerge` marks a reworded message `#, fuzzy` and `potext` refuses to load one,
so the window falls back to English until somebody has read the new sentence.

**The widget is told its language rather than asking.** `set_language` is the
whole of the public surface for it. An application has already decided what
language it is in, by whatever rule it keeps, and a tree disagreeing with the
window around it would be worse than an English one. The binary asks the
platform itself; the web build asks `navigator.language`, which is the browser's
answer to the question `potext` puts to the operating system elsewhere, asked in
the shell so that crate carries no `web-sys` for one line.

**Only a tag crosses a crate boundary, never a catalogue.** A catalogue would be
a type, and two versions of this crate in one graph would make it two.

`flyleaf-core::Kind::label` stays the canonical English: it is the model's name
for a kind, `tests/golden.rs` builds golden filenames from it, and a model crate
that knew about translation would be the wrong crate knowing it. The tree
translates a kind where it draws one. The kind names *are* translated — the
editor says *text* where TOML says *string*, which is the tell that they are the
window's words and not the format's.

`tree.rs` imports the lookup as `tr`, where every other file uses `t`; both
spellings are keywords in `flyleaf/po/update-po.sh`, and a catalogue built
without the alias listed looks healthy with every message in the tree missing.

## 11. Non-goals, and what would reopen each

**Schema-aware validation and completion**, the JSON Schema documents Taplo and
Tombi pull from SchemaStore. A real future feature and not a first one.

**A command-line tool.** What it would be: a typed `get`/`set` that writes the
file in place and leaves every other byte alone, for scripts and coding agents.
The nearest existing things are experimental, read-only, or silent about
comments, so the gap is real and narrow. The test for building it: if a CLI over
core's operation set is small, build it; if core's API turns out shaped around a
widget rather than around paths, skip it. Either way a separate crate, and never
a second parser or a second policy.

**Multi-file or project views.**

**An editable text mode.** §6 says why the source pane is read-only.

**Slipcase-specific behaviour**, which stays in slipcase-desktop and reaches the
widget through the policy hook §4 describes.

---

## License

MIT.
