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

Phase 1 is done when slipcase-desktop builds against the published crate and behaves exactly as before.

### Phase 2 — Generalize

Build the standalone app and grow the widget into a general TOML editor. Suggested order, adjust as needed:

1. Round-trip test corpus first: open→save with no edits is byte-identical, including TOML 1.1 features. Seed it with the valid cases from `toml-lang/toml-test`. Two things `toml_edit` does not preserve and core has to: a leading byte order mark is dropped, and CRLF line endings come back as LF (CRLF inside a multi-line string survives). Record both at load and reapply at save.
2. Full tree view: tables, arrays of tables, inline tables, key/value pairs; expand/collapse, selection. The lifted tree renders every row, and egui retains roughly 8.7 KB per row it has shown (measured in slipcase-desktop, which caps metadata at 256 KiB for that reason). A general editor gets handed `Cargo.lock` with thousands of entries, so decide here between collapsed-by-default sections and a virtualized tree before the tree grows.
3. Type-aware value editing: string, integer, float, bool, offset/local datetime, local date, local time, arrays, inline tables. Type changes are explicit actions.
4. Add/rename/delete keys and tables. Undo/redo.
5. Read-only source pane, synced with the tree, toggleable. slipcase-desktop doesn't have one because its users edit a small metadata document they don't think of as "a TOML file." A general TOML editor's users do — they hand-edit Cargo.toml — and the tree hides exactly what they care about: inline table vs `[header]`, quoting style, where comments went. The pane makes the editor's decisions visible, lets them verify round-trip fidelity without another tool, and is the only place comments are properly readable. Half the value is "what will save write," so a diff-against-disk view before saving is an acceptable alternative shape.

   Read-only in v1, deliberately: (a) editable text plus a tree sidebar is what Zed/VS Code already are, with better text editing than egui will give us — read-only keeps the tree as the product; (b) typing passes through unparseable TOML on every keystroke, so an editable pane needs a whole second layer of state handling for invalid intermediate states, error display, and resync without losing selection; (c) egui's `TextEdit` is basic — no highlighting, no line numbers, weak on large files — so a good editable pane means a real code-editor widget. Text→document sync itself is easy (`toml_edit` parses text); the cost is the UX around invalidity and the widget. If users want editable later, it's an addition, not a redesign.
6. Open/save/save-as via `rfd`; dirty state; unsaved-changes prompt. `flyleaf path/to/file.toml` opens that file.
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
