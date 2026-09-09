# Tommy Flyleaf — Design Document

What this application is, what it commits to, and what was measured to settle
each of them. `git log` is the record of *when* and in what order; this is the
record of *why*, and it is the authority on the application.

**Amend it in place where building contradicts it**, marked **Amended**, saying
what was measured. Do not smooth an amendment away: a design document that
quietly rewrites itself to match the code is worth nothing as a record.

It grew out of the kickoff prompt this repository started from on 2026-09-07,
which had become a chronology of phases nobody needed to read in order once
they were all done. What that file measured is here; when each phase happened
is in `git log`.

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

**One editor, not two that drift.** This is why the widget was extracted from
slipcase-desktop rather than copied. Where the editor needs behaviour it lacks,
the behaviour goes here and both applications get it.

## 3. What it commits to

**Round-trip fidelity.** Comments, key order, whitespace, quoting style and
table layout survive edits made elsewhere in the document. `toml_edit`, never
`toml`.

**TOML 1.1.0**, because the Slipcase specification requires it and the editor is
shared. Measured 2026-09-07 rather than assumed: `toml_edit` 0.25.13 carries
`+spec-1.1.0` and round-trips every 1.1 feature tried byte-identically — `\e`
and `\x` escapes, multi-line inline tables, trailing commas, comments inside
inline tables, optional seconds. It refuses non-ASCII bare keys, which is
correct; 1.1.0 final kept bare keys ASCII. Where the corpus finds a real gap the
path is upstream, not a fork and not a downgrade.

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

**Collapsed by default above two hundred entries**, and the threshold is a
reading decision rather than a memory one. Measured 2026-09-07: the window alone
is 140 MB resident; a `Cargo.lock` of 436 packages adds 25 MB with every section
open and under 1 MB with them closed; one five times larger adds 93 MB open.
About 10 KB per row egui has shown. Memory is tolerable either way, so what
decides is that a thousand open rows cannot be read. `flyleaf::open_all` opens
or closes everything on request. Virtualisation stays deferred until a
measurement says collapsing is not enough.

**The widget knows nothing about Slipcase.** Which keys are protected, and how a
protected string is displayed, come in as a policy the host supplies. That
reshaping is the one thing the extraction changed rather than moved.

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

**A field commits what was typed, not what it shows.** egui takes focus away on
Escape at the start of a frame, so a field left that way reseeded its text
before it could compare — a defect the key field had carried since
slipcase-desktop, found while building the comment slots.

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

The dialog runs on its own thread and is polled once a frame — the pattern
slipcase-desktop settled on after a blocking dialog froze its window. A close or
an open with unsaved changes is refused for the frame and asked about, with
Save, Don't save and Cancel; a save that fails keeps the prompt up with the
reason in it.

**The sibling rename is what the macOS App Sandbox refuses.** A grant covers the
file a person chose and not the directory holding it, so the rewrite has to wait
in `NSItemReplacementDirectory`, asked for with the target's URL so it lands on
the target's volume, and land through `replaceItemAtURL:`. Under `TMPDIR` it
fails with `EXDEV` for any file off the boot volume. That arm is in core's
`save_to`; slipcase-desktop's `staging.rs` is the same measurement made twice.

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
dotted keys interleaving two implicit tables being regrouped by table. The
corpus also caught the first `Document` accepting a file with two byte order
marks, stripping one itself and letting `toml_edit` strip the other.

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

**The size work, each step measured on one commit** after trunk's `wasm-opt
-Oz`: the default release profile gave 7.5 MB; a `web` profile with
`opt-level = "z"`, fat LTO, one codegen unit and `panic = "abort"` gave 6.4 MB;
eframe without its default `wgpu` renderer and with `glow` instead, on the wasm
target alone, gave 4.8 MB — 1.9 MB gzipped — and took wgpu-core and naga out of
the page. Dropping eframe's `accesskit` feature on wasm saved nothing, measured,
so it stays. What remains is egui, its default fonts and the editor, which is
the product.

The host serves `.wasm` as `application/wasm` unasked but compresses only text
types, so a `.gz` and a `.br` are written beside the wasm and a committed
`.htaccess` serves them by rewrite: 1.5 MB over the wire with brotli, the bytes
checked against the original by hash.

## 9. Packaging and the two stores

Debian for the Excelano apt repository, MSIX for the Microsoft Store, a signed
sandboxed bundle for the Mac App Store, crates.io, and a hosted WASM demo.
`packaging/` holds one directory per platform, cloned from slipcase-desktop's,
where most of these decisions were measured the first time; what follows is what
this repository learned that that one had not.

**Windows.** `AppxManifest.xml.in` was not valid XML: its opening comment spelled
a flag with two hyphens, which XML forbids inside a comment, and `makepri`
refused the substituted manifest with *Incorrect syntax was used in a comment*.
Nothing in the fleet parses that file until a Windows machine builds a package,
which is how a template written and reviewed on Linux got that far.

`uninstall.ps1` reported success and left the `UserChoice` behind, naming a
ProgID it had just deleted — the state its own comment calls killing the
extension outright. Explorer writes a *Deny SetValue* rule on that key so no
application can quietly take an extension over, and every delete that opens the
key for writing fails on it: `reg delete` says *Access is denied*, and .NET's
`DeleteSubKeyTree` reads the same failure as the key being missing and returns
quietly. It now deletes the name from the parent, which needs only DELETE and
works unelevated, reads every delete back, and refuses if a key survives.
`check-install.ps1` is the check that would have caught it: it installs, plants
a `UserChoice` the way Explorer writes one, deny rule and all, uninstalls, and
reads back that everything is gone. **slipcase-desktop's `uninstall.ps1` still
has the call this replaced and leaves the key behind on the same machine.**

`screenshot.ps1`'s six-second wait was too short on a cold start and blamed the
association rather than the clock: a 15 MB binary the disk has never read takes
longer than that to put up a window, and the same run a minute later took under
two seconds.

**macOS.** The private-symbol check refused the first bundle over two *public*
symbols — `CGDisplayCreateUUIDFromDisplayID` and its inverse, which winit links
through ApplicationServices and ColorSync's public headers — so the check reads
the headers rather than a list of names.

Rank Alternate is a tie-break and not a refusal: before the bundle was
registered `.toml` had no default handler on that Mac, and after it this
application was the default, because the alternatives declined to claim it.

The sandbox container is not empty and the privacy statement had to say so:
after one open through the panel, `~/Library/Containers/com.excelano.flyleaf`
holds a preferences plist.

**One blemish, seen in a store screenshot and not fixed:** a multi-line string
shows a missing-glyph box at each line break, because the tree edits every
string in a single-line field.

**The stores and crates.io are separate channels and their numbers need not
agree.** A `cargo publish` rebuilds no submission and a submission in review
holds no crate. What version the next release carries is decided when there is a
release to cut — a number written down in advance is a decision made before the
facts, and the one this document used to carry was wrong within two days.

**A catalogue is packaged only if it sits under the crate that reads it.**
`include_str!` reaching above a crate's own directory compiles locally and fails
in `cargo package`, which copies only what is under the crate root: the tarball
carries no catalogue and the publish dies verifying a build that cannot compile.
0.2.2's first tag was withdrawn for exactly that, before anything was published.
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

## 11. Non-goals, and what would reopen each

**Schema-aware validation and completion**, the JSON Schema documents Taplo and
Tombi pull from SchemaStore. A real future feature and not a first one.

**A command-line tool.** What it would be: a typed `get`/`set` that writes the
file in place and leaves every other byte alone, for scripts and coding agents.
Surveyed 2026-09-07: `toml-cli` is the nearest thing and is self-described
experimental, sets strings only, and prints rather than writes; `tomlq` is
read-only; `tomlpipe` says nothing about preserving comments. The gap is real
and narrow. The test for building it: if a CLI over core's operation set is
small, build it; if core's API turns out shaped around a widget rather than
around paths, skip it. Either way a separate crate, and never a second parser or
a second policy.

**Multi-file or project views.**

**An editable text mode.** §6 says why the source pane is read-only.

**Slipcase-specific behaviour**, which stays in slipcase-desktop and reaches the
widget through the policy hook §4 describes.

---

## License

MIT.
