# Windows packaging

Adapted from slipcase-desktop's `packaging/windows`, where every decision in
these files was measured and is recorded in its README; this one says what is
different here and how the pieces are used, and does not restate the rest.

    powershell -ExecutionPolicy Bypass -File packaging\windows\install.ps1
    powershell -ExecutionPolicy Bypass -File packaging\windows\install.ps1 -Default
    powershell -ExecutionPolicy Bypass -File packaging\windows\uninstall.ps1

## What is different

**`.toml` is a shared extension.** A Cargo.toml or a pyproject.toml is somebody
else's file and often already somebody else's association, so `install.ps1`
adds this application to the extension's Open With list and does not make it
the handler unless `-Default` is passed, which is what `packaging/linux/install.sh
--default` does on Linux. `uninstall.ps1` removes only what is ours: its own
ProgID from the list, the default only where it is this application's, and a
`UserChoice` only when it names this application. The Store package behaves
the same way without being asked, since an MSIX association is only ever an
Open With entry.

**The `UserChoice` key is deleted by name, not with the tree above it.**
slipcase-desktop's `uninstall.ps1` removes `FileExts\<ext>` whole, and that call
does not work: Explorer writes a *Deny SetValue* rule on `UserChoice` so no
application can quietly take an extension over, and every delete that opens the
key for writing — `.NET`'s `DeleteSubKeyTree`, `reg delete` — fails on it.
Measured here 2026-09-08, unelevated, on the real key: `reg delete` says *Access
is denied*, `DeleteSubKeyTree` reads the same failure as the key being missing
and returns quietly, and deleting the name from the parent works, because that
needs only DELETE and the rule beside the deny allows it. This repository does
the last; **slipcase-desktop has the first and leaves the key behind.**

**The certification baseline is empty** until a kit run fills it, which is the
fleet's rule. slipcase-desktop's carries `Blocked executables`, traced to the
standard library's batch-file spawn and `ShellExecuteW` under `opener`; this
application opens two links from its About box through the same route, so the
same finding is likely, and it is recorded in `build-msix.ps1` the day the kit
reports it.

**The uninstall entry is named apart from the Store package**, as `Tommy Flyleaf
(script install)`, so that a person with both can tell which to remove before
installing from the Store; the two registered at once put up Windows' picker on
every double-click.

## What is here

| File | What it is |
| --- | --- |
| `install.ps1`, `uninstall.ps1` | The per-user, no-toolchain route |
| `flyleaf.ico`, `assets/`, `listing/` | Built by `make-ico` from `packaging/linux/icons/flyleaf.svg`, the one drawing; committed because Windows has no step that rasterizes an SVG, and checked against the generator in `windows.yml` |
| `make-ico/` | The generator, its own package so nothing in it reaches the shipped binary |
| `flyleaf.manifest` | The DPI declaration `build.rs` hands the linker |
| `AppxManifest.xml.in`, `identity.psd1.example` | The Store package's manifest and the identity Partner Center assigns |
| `build-msix.ps1`, `check-imports.ps1`, `screenshot.ps1` | Build, the in-box import check, and the store screenshot |
| `check-install.ps1` | The two scripts above run against the registry and read back: what goes on comes off, a `UserChoice` naming this application goes with it, and another application's does not. Registers the real ProgID, so it is for a runner or a machine where that does not matter |

## The order, on the Windows machine

1. Reserve the name in Partner Center and copy `identity.psd1.example` to
   `identity.psd1` with what it shows under Product identity.
2. `cargo build --release`, then `check-imports.ps1`.
3. `install.ps1 -Default` and look: the window icon, the taskbar, Open With on
   a `.toml`, a double-click.
4. `build-msix.ps1 -SelfSign`, install the package as it prints, and look at
   the same four things from the package; then `-SelfSign -Certify` from an
   elevated prompt and record what the kit says in `$KNOWN_FINDINGS`.
5. `uninstall.ps1` before the Store copy is installed over it.
