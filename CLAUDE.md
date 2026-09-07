# CLAUDE.md

Guidance for Claude Code working in `flyleaf`. Read it before touching
anything. It is short because `PROMPT.md` is where the plan and the reasoning
live, and it is written from slipcase-desktop's `CLAUDE.md`, whose rules this
repository inherits rather than restates.

---

## What this is

**Tommy Flyleaf**, a structure-aware TOML editor. Two crates in one workspace:

- **`flyleaf-core`** — the document model over `toml_edit`, the edit
  operations, and what a save writes. No user interface. Heavily tested.
- **`flyleaf`** — the editor as an egui widget, which is what slipcase-desktop
  depends on, and the application binary, which is a thin shell around it.

It was extracted from the metadata editor in `excelano/slipcase-desktop`
(working copy `~/slipcase/slipcase-desktop`), and that application still uses
it: one editor across the excelano applications, not two that drift. Until the
extraction is complete, every change here is checked against slipcase-desktop
behaving as before; `PROMPT.md` lays out the slices and what each one's check
is, and slipcase-desktop's `tests/golden/` is the record it is checked against.

**`PROMPT.md` is the plan and it is alive.** When a decision changes, edit the
relevant section there in the same change. It is not a specification, and
where building contradicts it, amend it in place, marked **Amended**, saying
what was measured. Do not smooth an amendment away.

---

## Commands

    cargo build --all-targets
    cargo test
    cargo clippy --all-targets -- -D warnings    # must be silent
    cargo fmt --check
    cargo +1.95 build --all-targets               # the floor, measured

The workflow in `.github/workflows/ci.yml` runs all of these on every push,
and reads the floor out of `Cargo.toml` rather than carrying its own copy.

---

## Rules with no exceptions

**No unsafe.** Both crates are `#![forbid(unsafe_code)]`. A dependency that
carries unsafe on our behalf is fine, as `egui` does; unsafe in this
repository's own source is not, and lifting the `forbid` is a decision to take
with David.

**Nothing compiles C.** A crate that links a system library is fine; one that
builds C is not. `cargo tree -i cc` is not the check, because `cc` sits in
egui's tree as an ordinary Rust crate and always will. The outcome is what the
rule means: `ci.yml` refuses any `.o` or `.a` under the build directory, and
when the binary lands, `ldd` on it must name libc, libgcc_s and libm and
nothing else. `~/notes/pure_rust_preference.md` holds the fleet's stance and
its costs.

**One `toml_edit`.** `flyleaf-core`, `slpc` and slipcase-desktop must resolve
to a single `toml_edit` version, or `DocumentMut` becomes two types and
slipcase-desktop stops compiling. The workspace manifest says so beside the
number; move it and `slpc`'s together.

**The widget never mutates the document itself.** Every change goes through
`flyleaf-core`, so that undo/redo has one chokepoint. If the widget needs an
operation core lacks, the operation goes into core.

**Round-trip fidelity is the product.** Comments, key order, whitespace,
quoting style and table layout survive an edit made elsewhere in the document.
A change that costs any of that is a decision, not a side effect, and the
goldens exist to make it one.

---

## How to work

**Measure, do not assume.** The plan's first draft named `toml_edit`'s TOML
1.1 support as the likely first blocker; measured, it was not one. If you
assert something, run the thing that proves it, and record the measurement.

**Check that a test bites.** Break the fix deliberately, watch the test fail,
put it back. A regression test that passes against the defect it was written
for is worse than none.

**Every test's doc comment says what defect it would catch.**

**Comments say why.** A comment restating the line below it is noise. One
recording what was measured, what was rejected, or what breaks without the
line is why a file is readable a year later.

**Commit messages carry the reasoning.** Imperative subject under about 60
characters, prose body with the measurement and the alternatives rejected.

**The trailer block is one line.** A `Co-Authored-By` naming the model, and
nothing under it. Some harnesses append a `Claude-Session:` line carrying a
URL. This repository is public, so that is a private identifier written into a
permanent public record for no reader's benefit; slipcase-desktop has stripped
it from pushed history twice. Read what you are about to commit rather than
trusting what the harness composed.

**Never run `cargo fmt` in slipcase-desktop.** That repository is not
rustfmt-clean and has no fmt check; running it there during slice 3 rewrote
eight files the slice never touched and swamped a six-file change. This
repository is fmt-clean and CI holds it there; the other one is edited by
hand, in its own style.

**Prefer small, reviewable steps.** David verifies each slice of the
extraction against slipcase-desktop by hand; a slice that cannot be diffed
against what it replaced is too big.

---

## Layout

    Cargo.toml              the workspace: versions, the floor, the one toml_edit
    flyleaf-core/           the document model and the edit operations
    flyleaf/                the widget, and later the application binary
    .github/workflows/      ci.yml: the suite, clippy, fmt, the floor, and
                            the rule against compiling C
    PROMPT.md               the plan, alive
