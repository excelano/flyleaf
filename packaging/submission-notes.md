# Submission notes

What a store submission needs from a person and no file supplies: the answers a
form asks that no build can give, the notes an App Review or certification
reader is handed, and the reasoning behind the screenshots.

The listing text itself is not here. It is `store-listing.toml` beside this,
which `ship` checks before the tag and pushes to both stores on every release,
and what a release tells them changed is `release-notes.toml`. A field edited
in this file would reach nobody.

**No identity values.** The package name, publisher, family name and store id
Partner Center assigns are in `windows/identity.psd1`, which is not committed,
for the reason that file's committed example gives. The filled-in sheet for a
submission, identity and all, goes in `windows/SUBMITTING.local.md`, which
`.gitignore` reserves.

## The answers a form asks

**Category**

Developer tools > Utilities

**Copyright and trademark**

Copyright 2026 David M. Anderson. MIT licensed.

**Additional license terms**

MIT License. Full text: github.com/excelano/flyleaf/blob/main/LICENSE

**Pricing and availability**

Free, all markets, public.

**Age rating**

No user-generated content, no network access, no data collection, no advertising, no in-app purchases, no violence or mature content.

## App Review notes

Tommy Flyleaf is a TOML file editor. No account, no sign-in, no test credentials, and no network connection of any kind are needed to test it.

Sample files to test with are in the application's own public source repository, which is where they will stay: https://github.com/excelano/flyleaf/tree/main/packaging/samples

Direct downloads:

https://raw.githubusercontent.com/excelano/flyleaf/main/packaging/samples/every-kind.toml
https://raw.githubusercontent.com/excelano/flyleaf/main/packaging/samples/cargo-manifest.toml
https://raw.githubusercontent.com/excelano/flyleaf/main/packaging/samples/pyproject.toml
https://raw.githubusercontent.com/excelano/flyleaf/main/packaging/sample.toml

Open every-kind.toml first: it carries every kind of value TOML has, so every part of the editor is on screen at once. The other three are a Rust project manifest, a Python project file, and a short document with a comment in every position TOML allows one.

To exercise it: launch it and click Open, then choose one of those files; or drop the file on the Dock icon; or choose Tommy Flyleaf from Open With on it. Any other .toml file works as well — a Cargo.toml from a Rust project, or a file saved from TextEdit with a line such as: title = "hello". Change a value and save, and the file comes back with that value changed and every comment, key and line you did not touch exactly as it was, which is what this application is for. On launch with no file the window says so and offers the Open button; that empty state is expected and is not a failure to start.

The application declares the TOML document type and claims it at rank Alternate, not Owner: it is one editor for a format many applications open, and any application that claims .toml at a higher rank keeps double-clicks. On a Mac where nothing else claims the type, macOS will pick it, since there is no other candidate.

The App Sandbox is on with exactly two entitlements: the sandbox itself and read-write access to user-selected files. A save replaces the file the person chose, staged in the replacement directory macOS provides on the file's own volume and swapped in with one call, so it stays inside that grant. There is no network entitlement, and the application makes no network request; the About box has two links, to the product page and to the source repository, which open in the default browser.

The full privacy statement is at https://excelano.com/legal/#flyleaf and the complete source is at https://github.com/excelano/flyleaf.

## Notes for certification

The field a reviewer actually reads, and the one that decides whether a
submission comes back. It answers, in order, the four things that would
otherwise be guessed at: how to test an application that opens with nothing in
it, why it does not take the `.toml` default, what the certification kit's one
finding is, and why the no-network claim is checkable rather than asserted.

**The notes name GitHub and not `excelano.com/flyleaf/samples/`**, although
the page exists and says more, because the repository was serving the files the
hour the answer was needed and a site deploy is a step in someone's day. Naming
an address that 404s in the field a reviewer clicks is how one rejection
becomes two. The page is the better address once it is live, and the notes can
take it at the next submission.

The empty-window paragraph is not decoration. Launched with no file this
application shows a window with an Open button and a line of text, and a
reviewer who does not know a `.toml` is needed can read that as an application
that does not work. Nor is the sample-files line. Apple's reviewer had these
notes in their Mac wording, empty-window paragraph and all, and came back on
2026-09-10 asking for files anyway: telling a reviewer that any `.toml` will do
asks them to make one, and an address hands them four. The live Microsoft
listing was certified without that line and carries the old text until its next
submission.

Microsoft Store limit 2000 characters; this is 1964.

```
Tommy Flyleaf is a TOML file editor. No account, no sign-in, no test credentials, and no network connection of any kind are needed to test it.

Sample .toml files to test with, in this application's public source: github.com/excelano/flyleaf/tree/main/packaging/samples

To exercise it: launch it and click Open, or right-click any .toml file and choose Tommy Flyleaf under Open With. Any .toml file will do — a Cargo.toml from a Rust project, a pyproject.toml from a Python one, or a file typed into Notepad with a line such as: title = "hello". On launch with no file the window says so and offers the Open button; that empty state is expected and is not a failure to start.

The application claims .toml only as an Open With entry and never as the default handler. This is deliberate: .toml is a shared extension usually already owned by an editor or an IDE, and taking it would be taking somebody else's association.

The Windows App Certification Kit reports one finding against this package: Blocked executables, naming kernel32.dll!CreateProcessW and the string "cmd.exe". This comes from opening a web link. The About box has two links, to the product page and to the source repository, and the UI framework (egui/eframe) hands a clicked link to the system's default browser through the shell, which is what draws in that API and that string. The application itself spawns no process; the only use of the process API anywhere in the source is reading its own process id, in test code. The report's "rEG" and "dNx" lines are the scanner matching those letters inside a 15 MB binary and refer to no such program.

The application makes no network request. Its import table names nineteen libraries, all of them parts of Windows, and none of them is a networking library: no ws2_32.dll, no winhttp.dll, no wininet.dll. The full privacy statement is at https://excelano.com/legal/#flyleaf and the complete source is at https://github.com/excelano/flyleaf.

---
```

## Images

| What | Where | Built by |
| --- | --- | --- |
| Screenshot, 1366x768 | `dist/store/01-window.png` | `windows/shots.ps1` |
| Mac screenshot, light, 1440x900 | `dist/store/mac-01-window-light.png` | `macos/screenshot.sh --app "dist-universal/Tommy Flyleaf.app" --file packaging/sample.toml --out dist/store/mac-01-window-light.png`, with the Mac in Light |
| Mac screenshot, dark, 1440x900 | `dist/store/mac-01-window.png` | the same, with the Mac in Dark |
| Store logo, 1080x1080 | `windows/listing/store-logo-1080.png` | `windows/make-ico` |
| Store logo, 2160x2160 | `windows/listing/store-logo-2160.png` | `windows/make-ico` |

The screenshot is taken from the running application on the sample file, at the
Store's minimum size, by a script that refuses anything else; `shots.ps1` says
which file and which frame, and `screenshot.ps1` under it says what it measured
about window borders to get there. It is not committed —
it is a photograph of a build, and it is retaken when the interface changes.

## The Mac App Store

What is left here is Apple's own, and it is short because most of this
listing is now shared. The name, the subtitle, the promotional text, the
description, the keywords, the URLs and the release notes are top-level
sections above, written once and read by both lanes; the pricing and the age
rating are the same product's and are in the Microsoft section. What could not
move is below. The Mac screenshot is taken from a development-signed
bundle of the same commit, because a Store-signed one cannot launch off the
Store; `macos/screenshot.sh` refuses any size App Store Connect would. Two
captures, the light one first: the application follows the system appearance
and sets no theme of its own, so the Mac's appearance is switched for each
(`tell appearance preferences to set dark mode to false` in System Events,
and back), and the light one leads because the product page is set in cream.

**Name**

Tommy Flyleaf

**Category**

Developer Tools

**Export compliance**

Answered in the bundle: `Info.plist.in` declares `ITSAppUsesNonExemptEncryption` false. The application implements no cryptography.

**Privacy questionnaire**

Data not collected, every category. The statement above is the reason each answer is "no".
