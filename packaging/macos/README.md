# macOS

Adapted from slipcase-desktop's `packaging/macos`, where every decision in
these files was measured and is recorded in its README; this one says what is
different here and how the pieces are used, and does not restate the rest.

    cargo build --release
    ./packaging/macos/build-app.sh
    ./packaging/macos/build-app.sh --sign "Apple Development: ..."   # to measure the sandbox
    lsregister -f "dist/Tommy Flyleaf.app"

`lsregister` is not on `PATH`; `build-app.sh` prints the full line.

## What is different

**The type is imported, not exported.** This application does not own TOML,
so `Info.plist.in` imports a declaration for it under `io.toml.toml`, chosen in
the format's own domain because macOS declares none, and claims it with rank
`Alternate` rather than `Owner`: one editor among the many that open a `.toml`,
and never the default over an application that claims the type at a higher
rank. Measured 2026-09-08 on this Mac: nothing else declared a TOML type at
all, `.toml` had no default handler before this bundle was registered, and
after it this application was the default, because rank decides among
claimants and there was no other. The identifier was measured the same day
against everything installed, `Info.plist.in` records how, and `io.toml.toml`
stands.

**The bundle has a space in its name**, `Tommy Flyleaf.app`, because that is
the product name and it is what a person sees in Applications and the Dock.
Every script here quotes its path; anything new that takes the bundle's path
has to as well.

**The save does not go through a sibling file.** Under the sandbox a person's
grant covers the file they chose and not its directory, so `flyleaf-core`'s
`save_to` has a macOS arm that stages the rewrite in the directory macOS
provides for replacements, on the file's own volume, and lands it with
`replaceItemAtURL:`. The measurement is slipcase-desktop's `staging.rs`; the
bindings are safe functions, so the core crate keeps `forbid(unsafe_code)`.

**The double-click handler is the application's one `unsafe`.**
`flyleaf/src/opened_document.rs` is slipcase-desktop's module with the class
renamed, installed at `applicationWillFinishLaunching:`, the only moment of
three that catches both a cold launch and a document dropped into a running
window. `CLAUDE.md` records the decision that lifted `forbid` to `deny` in the
application crate for it; `flyleaf-core` stays `forbid`.

## What is here

| File | What it is |
| --- | --- |
| `Info.plist.in` | The bundle's property list, with the version substituted |
| `Flyleaf.entitlements` | The sandbox and user-selected files, for a development signature; the Store signature adds the identifiers from the profile |
| `build-app.sh` | Assembles, signs, checks for private symbols, and builds the Store package with `--store` |
| `check-install.sh` | Asks an installed bundle what it is, on the machine it is on |
| `screenshot.sh`, `window-probe.swift` | The store screenshot, and the window check `apple-silicon.yml` runs |

## The order, on the Mac

1. `cargo build --release`, `build-app.sh --sign` with the Apple Development
   identity, `lsregister -f`, and look: the Dock icon, a `.toml` under Open
   With, a double-click delivering the document, a save on a file chosen in the
   open panel, and one on a file on a second volume. **Done 2026-09-08** on an
   Intel Mac running macOS 15.7.9; `PROMPT.md` has what each one measured.
2. `lsregister -dump | grep -i toml`, and settle the identifier. **Done the
   same day**, above.
3. The two `MACOSX_DEPLOYMENT_TARGET=12.0` builds and `build-app.sh --store`
   with the profile, once the name is reserved in App Store Connect and the
   profile downloaded. The two builds are done and `build-app.sh --universal
   --sign` passed the floor check on both slices; `--store` waits on the
   profile.
4. `check-install.sh` against the installed copy, then TestFlight.

**What the sandbox writes, measured.** After a launch, an open through the
panel and a save, `~/Library/Containers/com.excelano.flyleaf` holds one
preferences plist with four keys, all the open panel's own: its size, two
window frames, and a bookmark of the last folder it showed. `lsof -i` on the
running process lists nothing. `privacy-entry.html` says both.

**Driving the window from a script.** egui exposes no accessibility elements,
so System Events cannot click a widget, and `click at` is refused even where
`set position` is allowed. A mouse event posted through `CGEvent` from a
process the terminal is responsible for lands; the save checks above were
made that way, each one diffing the saved file against `sample.toml` and
finding exactly the one line changed.
