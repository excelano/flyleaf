# Tommy Flyleaf — project kickoff

You are starting a new project in this empty folder. Read this whole file, then work through the open questions at the bottom with me before writing code.

This document is a starting position, not a spec. Everything in it is open to revision as we go. When you think something here is wrong, or a better option appears once you've seen the code, say so and propose the change — then update this file so it stays the current statement of the plan. Treat it as living: we will edit it together throughout the project.

## What it is

**Tommy Flyleaf** is a standalone, structure-aware TOML editor. Desktop GUI, cross-platform, Rust with egui/eframe. It grows out of the metadata editor inside `slipcase-desktop` (excelano/slipcase-desktop, working copy `~/slipcase/slipcase-desktop`), which currently edits only the TOML member of a `.slpc` container. That editor reaches `toml_edit` through the `slpc` library's re-export (excelano/slpc-rust), which is the only thing the library repo has to do with it.

The gap it fills: existing TOML tooling is editor plugins/LSPs (Taplo, Tombi), path-based CLIs (tomlpipe, dasel, tomlq), or one-off config editors tied to a specific app (mise). There is no general-purpose interactive editor that treats the document as a tree, edits values type-aware, and preserves comments and formatting on save.

## Naming

- Display name: **Tommy Flyleaf**
- Crate and binary: `flyleaf`; core library: `flyleaf-core`
- GitHub: `excelano/flyleaf`
- The name reads as a person's name (a bookbinder's apprentice). Lean into that later in the About box and icon.

## The approach: extract first, generalize second

This is a refactor of slipcase-desktop before it is a new app. The editor becomes a shared component that slipcase-desktop consumes; Tommy Flyleaf is a thin standalone shell around that same component. One TOML editor across the excelano apps, not two that drift.

Current plan for shape (adjustable):

- **Own repo**, `excelano/flyleaf`. slipcase-desktop takes a dependency on it — path or git during extraction, crates.io once published.
- **`flyleaf-core`**: document model over `toml_edit`, edit operations, serialization. No UI. Heavily tested.
- **`flyleaf`**: the egui widget as a library, plus the app binary in the same crate. The widget is what slipcase-desktop depends on; `main` is a small shell.

### Phase 1 — Extraction (zero behavior change in slipcase-desktop)

1. Read the existing metadata editor in slipcase-desktop. Report what's worth lifting as-is, what needs reshaping to be general, and what should be rewritten. We decide together before you move anything. Done 2026-09-07; the findings are in the Decisions section below.
2. Move the editor into `flyleaf`/`flyleaf-core` in this repo. The one reshaping this needs: the widget's protected-key rule (`is_protected`) and its escaped display of protected strings hard-code Slipcase's two required keys through `slpc`. They become a policy hook the widget takes and slipcase-desktop supplies. Everything else is a move with renamed imports.
3. Point slipcase-desktop at it. A git dependency pinned by revision, not a path, so slipcase-desktop's three platform workflows keep building. Its UI must be functionally identical afterwards.
4. Publish `flyleaf-core` and `flyleaf` 0.1 to crates.io and flip slipcase-desktop to the registry version. This was Phase 2's last step; it moved here because crates.io refuses a git or path dependency without a version, so slipcase-desktop cannot release again until flyleaf is published, and Phase 2 is too long to hold that.

Phase 1 is done when slipcase-desktop builds against the published crate and behaves exactly as before. **Done 2026-09-07**, in one day, all eight slices, with the goldens holding through every one.

Built in eight slices, each leaving slipcase-desktop building and behaving as before, each with its own check. "Behaving as before" is a claim the existing suite cannot make on its own: its tests check behaviours, the corpus runner checks only whether a tree appears, and the rest is `CHECKLIST.md`. Slice 0 turns the claim into something that fails.

0. **Regression baseline, in slipcase-desktop, before anything moves.** Done 2026-09-07. Two golden tests committed there. A render golden: a fixed set of documents (the every-type fixture, a comment-heavy one, a few corpus metadata members copied in) rendered headlessly at a fixed width and the four scales the existing layout test uses, every text shape dumped in order with its text and position, compared against a committed file. An edit golden: a scripted sequence over the same documents, setting every scalar type, adding each kind, renaming and removing at the root, in a table, in an inline table and in an array of tables, with the serialized result compared against a committed file; that is "what a save keeps" as a test rather than a checklist item. Plus the recorded baseline: test count, corpus count, and the pure-Rust check's output. The goldens run unchanged through every later slice and regenerate only by decision; after slice 5 they move to flyleaf as well, as the widget's own suite under a neutral policy. The render golden is tied to egui's glyph metrics and so to egui 0.36, which Phase 1 does not bump. Hand checks stay: `CHECKLIST.md`'s save items after slices 4, 5 and 7 on Linux, and one pass on Windows and macOS before 7.
1. **Workspace skeleton.** Done 2026-09-07. `flyleaf-core` and `flyleaf` at the agreed versions, a `CLAUDE.md` inherited from slipcase-desktop's, a Linux workflow with build, test, clippy and an MSRV job at 1.95. No code. Check: empty-green tests, silent clippy.
2. **Core gets the operations.** Done 2026-09-07. `NewKey`, add, remove and rename for `Table` and `InlineTable`, `set_value`, with their tests, copied into `flyleaf-core`; slipcase-desktop keeps its copy for one slice. Check: core tests green, and a diff against `lib.rs` lines 352-553.
3. **slipcase-desktop takes core.** Done 2026-09-07. Git dependency pinned by revision; the duplicated operations and tests leave `lib.rs`; `tree.rs` imports from `flyleaf_core`. Check: its tests, clippy and the corpus runner agree, `cargo tree -d` shows one `toml_edit`, the pure-Rust outcome check finds nothing new.
4. **Policy hook, in place.** Done 2026-09-07, hand check passed on Linux. Inside slipcase-desktop's `tree.rs`, before it moves, `render` takes a policy answering whether a path is protected and how a protected string is displayed; slipcase-desktop's implementation wraps the `slpc` constants and `display_name`, and the two protection tests move to its side. Reshaping here rather than during the move keeps slice 5 a move that can be diffed against the deleted file. Check: tests green, and `CHECKLIST.md`'s metadata items by hand, at least protected keys and the bidi payload name.
5. **Tree moves to flyleaf.** Done 2026-09-07. `tree.rs` and its tests become the widget; slipcase-desktop deletes its copy and calls `flyleaf::render` with its policy. Check: tests green in both repos, the corpus runner, `CHECKLIST.md`'s metadata items by hand on Linux; Windows and macOS come through slipcase-desktop's own workflows, the widget having no platform arms.
6. **The shell binary.** Done 2026-09-07. `flyleaf path.toml` opens the file and shows the tree; edits stay in memory and saving is Phase 2 item 6. Check: it runs, and nothing in slipcase-desktop changes.
7. **Publish 0.1 and flip slipcase-desktop.** Done 2026-09-07: `flyleaf-core` and `flyleaf` 0.1.0 on crates.io, tag `v0.1.0`, slipcase-desktop on the registry version with its publish dry run passing. `flyleaf-core`, then `flyleaf`, then slipcase-desktop on `flyleaf = "0.1"`. Check: `cargo install flyleaf` works and slipcase-desktop's `cargo publish --dry-run` passes, which it cannot between slices 3 and 7.

### Phase 2 — Generalize

Build the standalone app and grow the widget into a general TOML editor. Suggested order, adjust as needed:

1. Round-trip test corpus first: open→save with no edits is byte-identical, including TOML 1.1 features. Seed it with the valid cases from `toml-lang/toml-test`. Two things `toml_edit` does not preserve and core has to: a leading byte order mark is dropped, and CRLF line endings come back as LF (CRLF inside a multi-line string survives). Record both at load and reapply at save.

   **Done 2026-09-07.** The TOML 1.1.0 cases of toml-test are vendored under `flyleaf-core/tests/toml-test/`, 220 valid and 494 invalid, and `tests/roundtrip.rs` holds every valid one to a byte-identical render through `flyleaf_core::Document` and every invalid one to a refusal. Measured: there was a third thing `toml_edit` drops, a missing final newline, and `Document` records all three. What remains is `toml_edit`'s and cannot be put back from outside the parser; seven cases, named in the test with the reason each, which asserts they still differ so that a fix upstream is noticed rather than absorbed. Two shapes, both candidates for upstream issues against `toml-rs/toml`: a non-leaf dotted-key segment or a table-header segment is rendered from the key's first definition rather than from each occurrence, losing its quoting and the whitespace before its dot (`"count".c` after `count.a` comes back `count.c`; `[a.'b']` after `['a']` comes back `['a'.'b']`); and dotted keys that interleave two implicit tables are regrouped by table. The corpus also caught a defect in the first `Document`: it stripped one byte order mark and let `toml_edit` strip a second, so a file with two was accepted.
2. Full tree view: tables, arrays of tables, inline tables, key/value pairs; expand/collapse, selection. The lifted tree renders every row, and egui retains roughly 8.7 KB per row it has shown (measured in slipcase-desktop, which caps metadata at 256 KiB for that reason). A general editor gets handed `Cargo.lock` with thousands of entries, so decide here between collapsed-by-default sections and a virtualized tree before the tree grows.

   **Done 2026-09-07, collapsed-by-default.** Measured first: the window alone is 140 MB resident, a `Cargo.lock` of 436 packages adds 25 MB with every section open and under 1 MB with them closed, and one five times larger adds 93 MB open. About 10 KB per row shown, so memory is tolerable either way and reading is what decides: above 200 entries every section starts closed, below it every section starts open, and `flyleaf::open_all` opens or closes them all on request. Selection is the row holding keyboard focus, highlighted and returned from `render` as its path; the shell shows it. Virtualization stays deferred until a measurement says collapsing is not enough.
3. Type-aware value editing: string, integer, float, bool, offset/local datetime, local date, local time, arrays, inline tables. Type changes are explicit actions.

   **Done 2026-09-07.** `flyleaf_core::Kind` names all eleven kinds, the four datetime shapes told apart, and every row and section carries a small button reading its kind whose menu offers the others the container can hold, enabled only where `flyleaf_core::convert` says the value reads as the target: a scalar to text always, text to whatever it parses as, an integer to a float and a whole float back, a datetime to another shape by dropping or filling with the epoch, midnight or UTC, any value to a one-element array and back. A boolean does not become a number, a float with a fraction does not become an integer, and nothing becomes an inline table; those items are shown disabled rather than refused after the fact. A table becomes an inline table and back in its own place with its comment carried across, which is the one change that is about layout rather than type. Arrays are sections of index-named rows with add, remove and the same kind menu, and a new element takes the style of the one before it so a multi-line array stays one. The goldens were regenerated for the kind buttons and the five new kinds the edit script adds.
4. Add/rename/delete keys and tables. Undo/redo.

   **Done 2026-09-07.** Add, rename and delete were there for tables and inline tables and item 3 added them for arrays. Undo is in `flyleaf_core::Document`: a copy of the rendered text per step, a step being what changed between two `record` calls, with calls naming the same row coalesced so a word typed into a field undoes as a word; undo parses the previous text back, which loses nothing. The shell records after every frame, grouped by the selected row, takes Ctrl+Z and Ctrl+Shift+Z (or Ctrl+Y) before the tree draws so a focused field cannot answer with its own undo, and calls `flyleaf::forget_typing` first, because a key field commits its buffer on blur and would otherwise rename the row back. `Document` also owns the edited mark now; slipcase-desktop's `Opened` can take it from there at the next bump.
5. Read-only source pane, synced with the tree, toggleable. slipcase-desktop doesn't have one because its users edit a small metadata document they don't think of as "a TOML file." A general TOML editor's users do — they hand-edit Cargo.toml — and the tree hides exactly what they care about: inline table vs `[header]`, quoting style, where comments went. The pane makes the editor's decisions visible, lets them verify round-trip fidelity without another tool, and is the only place comments are properly readable. Half the value is "what will save write," so a diff-against-disk view before saving is an acceptable alternative shape.

   **Done 2026-09-07, the synced view.** A right-hand panel, on by default and toggled with a Source button, draws `Document::render` one monospace line per row inside a scroll area that lays out only the visible rows, so a five-thousand-line lockfile costs what fits on screen. The row the tree has focus in is highlighted and scrolled a third of the way down the frame the selection changes, and left alone after. The sync goes through `Document::lines_of`: `DocumentMut` keeps no positions once it can be edited, so the rendered text is parsed again as `toml_edit::Document`, which does, and the selected path's span becomes a line range; a row is its key to the end of its value, a table or an array-of-tables element its header line. That parse runs only when the selection or the document changes. Selection and copy in the pane, and the diff-against-disk mode, are still to come.

   Read-only in v1, deliberately: (a) editable text plus a tree sidebar is what Zed/VS Code already are, with better text editing than egui will give us — read-only keeps the tree as the product; (b) typing passes through unparseable TOML on every keystroke, so an editable pane needs a whole second layer of state handling for invalid intermediate states, error display, and resync without losing selection; (c) egui's `TextEdit` is basic — no highlighting, no line numbers, weak on large files — so a good editable pane means a real code-editor widget. Text→document sync itself is easy (`toml_edit` parses text); the cost is the UX around invalidity and the widget. If users want editable later, it's an addition, not a redesign.
6. Open/save/save-as via `rfd`; dirty state; unsaved-changes prompt. `flyleaf path/to/file.toml` opens that file.

   **Done 2026-09-07.** `Document::from_path` and `Document::save_to` in core behind a default `fs` feature, the save written to a sibling and renamed over the file so a failure anywhere leaves the original whole. The shell has Open, Save and Save as with Ctrl+O, Ctrl+S and Ctrl+Shift+S taken before the tree draws; the dialog runs on its own thread and is polled once a frame, the pattern slipcase-desktop settled on after a blocking dialog froze its window; the title carries a mark while there is something unsaved; a close or an open with unsaved changes is refused for the frame and asked about, with Save, Don't save and Cancel, and a save that fails keeps the prompt up with the reason. **For Phase 3:** the sibling rename is what the macOS App Sandbox refuses for a file a dialog granted, and slipcase-desktop's `src/staging.rs` already holds the `NSFileManager` arm that answers it; that arm moves into core's `save_to` when the Mac App Store build is made.
7. File association.
8. WASM target via the eframe web build (browser picker/download for files).
9. Release the result. 0.1 went out at the end of Phase 1; this is whatever version the changes above add up to, with slipcase-desktop bumped to it.

### Phase 3 — Packaging

Debian package for apt, Microsoft Store (MSIX), Mac App Store (signed, sandboxed bundle), crates.io (`cargo install flyleaf`), hosted WASM demo. Don't make choices in Phases 1–2 that block these — in particular, design file access around platform dialogs, since macOS sandboxing and MSIX both constrain arbitrary path access.

slipcase-desktop's packaging already paid for most of this once, and its `packaging/` directory and `RELEASE.md` are the starting point. One item that will follow the widget: its `Cargo.toml` pins a forked `winit` that strips a private Apple symbol App Store review rejects (Guideline 2.5.1). The flyleaf app needs the same patch until a winit release carries the `private-apple-apis` gate.

## Current commitments

These are the things I currently feel strongly about. They are still up for discussion — if one is wrong or in tension with another, raise it — but changing one is a decision we make explicitly, not a drift.

- **Round-trip fidelity.** Comments, key order, whitespace, quoting style, and table layout survive edits elsewhere in the document. `toml_edit`, not `toml`.
- **TOML 1.1.0.** The slipcase spec requires it, so the shared editor must read and write it. Measured 2026-09-07 rather than assumed: `toml_edit` 0.25.13 carries `+spec-1.1.0` and parses and round-trips byte-identically every 1.1 feature tried (`\e` and `\x` escapes, multi-line inline tables, trailing commas, comments inside inline tables, optional seconds). It refuses non-ASCII bare keys, which is correct; 1.1.0 final kept bare keys ASCII. Not a blocker. The fidelity gaps it does have (BOM, CRLF) are not 1.1 matters and are handled in Phase 2 item 1. Should the toml-test corpus find a real gap, the path is still contributing upstream, not forking or downgrading.
- **Structure-aware, not a text editor.** The tree is the primary view.
- **The GUI is a single binary with no CLI subcommands.** On Windows an executable is either GUI-subsystem or console-subsystem, and apps that try to be both end up with AttachConsole hacks; store builds aren't on PATH anyway. If we want a CLI (a comment-preserving `flyleaf get`/`set` over the same round-trip engine, aimed at scripts and AI coding agents), it is a separate `flyleaf-cli` binary in this workspace over `flyleaf-core` — not this binary. Deferred, not rejected; see below.
- **Pure Rust, cross-platform, WASM-capable.**
- **One editor.** slipcase-desktop and Tommy Flyleaf share the widget. No forks.

## Out of scope for v1 (revisit later)

- Schema-aware validation and completion (the JSON Schema documents Taplo/Tombi use for Cargo.toml, pyproject.toml, etc. via SchemaStore) — a real future feature, not v1
- `flyleaf-cli` — decide once `flyleaf-core`'s API has settled through Phase 2. What it would be: a typed `get`/`set` that writes the file in place and leaves every other byte alone, for scripts and coding agents. Surveyed 2026-09-07: `toml-cli` is the nearest thing and is self-described experimental, sets strings only, and prints rather than writes; `tomlq` is read-only; `tomlpipe` says nothing about preserving comments. The gap is real but narrow. The test for building it: if a CLI over core's Phase 2 operation set is small, build it; if core's API turns out shaped around a widget rather than around paths, skip it. Either way it is a separate crate and never a second parser or a second policy.
- Multi-file or project views
- Editable text mode (see Phase 2 item 5 for why)
- Slipcase-specific behavior — that stays in slipcase-desktop, which calls into the widget

## Decisions — settled 2026-09-07

The kickoff's open questions, answered. Reopen any of them by editing this section.

**What the existing editor is.** `src/tree.rs` in slipcase-desktop (983 lines, 14 tests) renders the document with one arm per TOML type and extracts comments from decor, including the trailing ones. `src/lib.rs` holds the edit operations: `NewKey`, add, remove and rename for both `Table` and `InlineTable`, and `set_value`, which keeps decor; the rename rebuilds the table so the key keeps its position and comments. Edited detection compares `to_string()` against the parsed baseline. `main.rs` calls the tree once, inside a `ScrollArea`. Lift the operations, the renderer, the datetime buffer, the commit-on-blur key field, the hour-zero repair and the layout tests as they are. Reshape only the protection policy (Phase 1 step 2). Rewrite nothing. The edited baseline stays in slipcase-desktop for Phase 1 and becomes a core `Document` type in Phase 2, when undo/redo needs it.

**Two crates, confirmed.** `flyleaf-core` starts as about 200 lines of operations and their tests, which is thin; Phase 2 fills it with the baseline and dirty state, undo/redo, explicit type conversion, BOM and line-ending preservation, and the corpus. The rule that makes the split worth having: the widget never calls a `toml_edit` mutation directly. Every change goes through core, so undo/redo has one chokepoint. `flyleaf` is the widget library plus the app binary; the binary's own dependencies go behind a default `app` feature the moment one appears that slipcase-desktop does not already carry.

**One `toml_edit`.** `flyleaf-core`, `slpc` and slipcase-desktop must resolve to a single `toml_edit` version or `DocumentMut` becomes two types and slipcase-desktop stops compiling. Both are on `0.25`. Say so in both manifests.

**License MIT**, matching every excelano code repository this widget compiles into.

**Versions.** eframe and egui 0.36 (0.36.1 is current and what slipcase-desktop uses). MSRV 1.95, which is eframe's own floor, measured the way slipcase-desktop's manifest does rather than trusted. Edition 2021, to match the source the code moves from. `toml_edit` 0.25, in lockstep with `slpc`. rfd 0.17 when Phase 2 needs it. `flyleaf`, `flyleaf-core` and `flyleaf-cli` were all free on crates.io on 2026-09-07 and `excelano/flyleaf` did not yet exist.

**Rules inherited from slipcase-desktop**, not restated: `#![forbid(unsafe_code)]`, nothing compiles C, measure rather than assume, every test's doc comment says what defect it catches, comments say why, commit messages carry the reasoning and the trailer is one line. This repo's `CLAUDE.md` starts from that one.

## How we work

- Keep this file current. When we change a decision, edit the relevant section here in the same change.
- Prefer small, reviewable steps. Phase 1 especially: I want to see the extraction happen in stages I can verify against slipcase-desktop.
- When unsure, ask. When you have a recommendation, give it with the reasoning, not just the options.
