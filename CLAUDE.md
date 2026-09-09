# CLAUDE.md

Guidance for Claude Code working in `flyleaf`. Read it before touching
anything. It is short because `DESIGN.md` is where the reasoning lives, and it
is written from slipcase-desktop's `CLAUDE.md`, whose rules this repository
inherits rather than restates.

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
behaving as before, and slipcase-desktop's `tests/golden/` is the record that
is checked against.

**`DESIGN.md` is the authority on this application and it is alive.** When a
decision changes, edit the relevant section there in the same change. Where
building contradicts it, amend it in place, marked **Amended**, saying what was
measured. Do not smooth an amendment away: a design document that quietly
rewrites itself to match the code is worth nothing as a record.

---

## The language it draws in

**The catalogues live inside the crate that reads them and not beside the
workspace**, because `include_str!` reaching above a crate's own directory
compiles here and fails in `cargo package`: the tarball carries only what is
under the crate root, so the publish dies verifying a build with no catalogue.
0.2.2's first tag was withdrawn for exactly that.

German, since 2026-09-09, through [`potext`](https://crates.io/crates/potext) —
the reader written in slipcase-desktop and made a crate so that this repository
could use it. `flyleaf/po/` holds the catalogues and the two commands that keep them
current.

**The widget does not ask what language it is in; it is told.**
`flyleaf::set_language` is what a host application calls, and the application
binary calls it too after asking the platform itself. An application has
already decided what language it is in, and a tree that disagreed with the
window around it would be worse than an English one. Only a language tag ever
crosses a crate boundary — never a catalogue, which would be a type, and two
versions of this crate in one graph would make it two.

**`tree.rs` imports the lookup as `tr` and every other file uses `t`.** That
file has called a table `t` since it was written, in a dozen bindings and two
signatures. `flyleaf/po/update-po.sh` lists both spellings as keywords, and leaving the
alias out of that list silently dropped every message in the tree from a
catalogue that still looked healthy.

**Run `flyleaf/po/pseudo.sh` before writing a translation, not after.** It found three
things here that the German pass had missed: an `Add` button and a placeholder
that never went through the lookup, and two kind badges too wide for the pane.
`flyleaf-core::Kind::label` stays the canonical English — the golden filenames
are built from it — and `tree.rs` translates a kind where it draws one.

---

## Commands

    cargo build --all-targets
    cargo test
    ./flyleaf/po/update-po.sh                             # after changing any sentence a person reads
    ./flyleaf/po/pseudo.sh                                # then run a debug build with POTEXT_LANG=en-x-pseudo
    cargo clippy --all-targets -- -D warnings    # must be silent
    cargo fmt --check
    cargo +1.95 build --all-targets               # the floor, measured
    cargo check -p flyleaf --no-default-features  # the widget alone
    cargo run -- path/to/file.toml                # the application
    ./packaging/linux/install.sh                  # Linux desktop integration
    cargo check -p flyleaf --target wasm32-unknown-unknown   # the web arm
    trunk build --release                         # the web page, into dist/
    ./packaging/debian/build-deb.sh               # the .deb, after a release build
    ./packaging/linux/check-libraries.sh          # what the running app opens, against Depends;
                                                  # needs a display, so run it after touching a dependency
    cargo clippy -p flyleaf --all-targets --target x86_64-pc-windows-msvc -- -D warnings
    cargo clippy --workspace --all-targets --target aarch64-apple-darwin -- -D warnings
                                                  # the two platform halves, type-checked from here;
                                                  # windows.yml and apple-silicon.yml run them
    powershell -File packaging\windows\install.ps1       # on Windows; README there has the order
    powershell -File packaging\windows\check-install.ps1 # what it registers, on and off again;
                                                  # windows.yml runs it, and it takes the real
                                                  # .toml registration over while it does
    ./packaging/macos/build-app.sh                # on a Mac; README there has the order

The workflow in `.github/workflows/ci.yml` runs all of these on every push,
and reads the floor out of `Cargo.toml` rather than carrying its own copy.

---

## Rules with no exceptions

**No unsafe, with one exception.** `flyleaf-core` and the widget are
`#![forbid(unsafe_code)]`. A dependency that carries unsafe on our behalf is
fine, as `egui` does. The application binary carries `deny` instead, with one
`allow`, on `src/opened_document.rs`: macOS delivers a double-clicked document
as an Apple Event, receiving one means defining an Objective-C class, and
`objc2` cannot do that without `unsafe`. It is slipcase-desktop's module with
the class renamed, and the decision to take it was made with David on
2026-09-08. A second module wanting the allow is a decision, not a precedent.

**Nothing compiles C.** A crate that links a system library is fine; one that
builds C is not. `cargo tree -i cc` is not the check, because `cc` sits in
egui's tree as an ordinary Rust crate and always will. The outcome is what the
rule means: `ci.yml` refuses any `.o` or `.a` under the build directory, and
`ldd` on the binary must name libc, libgcc_s and libm and nothing else.
`.cargo/config.toml` links the C runtime into the Windows binary for the
reason it records. `~/notes/pure_rust_preference.md` holds the fleet's stance
and its costs.

**The GUI is a single binary with no subcommands.** On Windows it is a
GUI-subsystem executable and prints nothing, so an error is shown in the
window. Anything scriptable is `flyleaf-cli`'s, if that is ever built, and
never this binary's.

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
    flyleaf/                src/lib.rs, src/tree.rs and src/source.rs are
                            the widget, the tree and the source pane;
                            src/main.rs is the application, behind the
                            default `app` feature; tests/golden/ is the record
    .github/workflows/      ci.yml: the suite, clippy, fmt, the floor, the
                            rule against compiling C and the web bundle;
                            windows.yml and apple-silicon.yml: the suite on
                            those platforms, the import check, the icon
                            check, and a document opened through Launch
                            Services; publish-crate.yml on a tag
    flyleaf/index.html      the page the web build draws into; Trunk.toml
                            at the root says how it is built
    packaging/linux/        the desktop entry, the icon, install.sh,
                            uninstall.sh and check-libraries.sh; the icon in
                            icons/ is the one drawing every platform's comes from
    packaging/windows/      the two install scripts and the check that they go
                            on and come off cleanly, the MSIX build, the import
                            check, the committed .ico and assets and make-ico
                            that builds them; README says what differs from
                            slipcase-desktop's
    packaging/macos/        the bundle build, the property list, the
                            entitlements, the install check and the window
                            probe; README says what differs
    packaging/sample.toml   the file the screenshots and the hosted demo open
    packaging/privacy-entry.html
                            the privacy statement, copied verbatim into
                            excelano.com/legal; edit here first
    packaging/store-listing.md
                            the store listing copy and the notes a reviewer
                            reads; edit here first. The identity a store
                            assigns is not here, and does not go here
    packaging/debian/       control.in, the changelog, the manual page and
                            build-deb.sh; version.sh beside them reads the
                            one version in Cargo.toml in every spelling
    DESIGN.md             what this is and why, alive
    UPSTREAM.md             what toml_edit does to a document, each finding's
                            reproduction, and whether it is filed
