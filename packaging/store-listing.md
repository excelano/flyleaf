# Store listing

The copy a store listing is filled in from, kept here for the reason
`sample.toml` and `privacy-entry.html` are: it is written from the product
rather than beside it. Nearly every line below is a claim about the code —
eleven kinds, TOML 1.1.0, a save that replaces the file in one step, an import
table with no networking library in it — and a claim like that goes stale
silently in a web form nobody diffs. **Edit here first, then paste.**

**No identity values.** The package name, publisher, family name and store id
Partner Center assigns are in `windows/identity.psd1`, which is not committed,
for the reason that file's committed example gives. The filled-in sheet for a
submission, identity and all, goes in `windows/SUBMITTING.local.md`, which
`.gitignore` reserves.

Paragraphs are single lines on purpose. A store field is a web form, and text
hard-wrapped at 80 columns pastes into one with the wraps still in it.

Submitted to the Microsoft Store 2026-09-08, at 0.2.1. The Mac App Store cut
was written the same day, from a Mac; its section is at the end.

---

## Everything a listing needs

**Product name**

Tommy Flyleaf

**Short description** (Microsoft Store limit 1000; this is 145)

Edit a TOML file as a tree, every value by its kind, and save it with the comments, key order and formatting of everything you did not edit kept.

**Description** (Microsoft Store limit 10000; this is 1705)

Tommy Flyleaf is a structure-aware TOML editor. It opens a file, shows it as a tree beside what a save would write, and edits every value by its kind.

A save keeps every comment, the key order, the whitespace and the quoting of everything you did not edit. A Cargo.toml or a pyproject.toml comes back the way you left it, one value different.

What it does:

• Shows the document as a tree, one renderer per TOML type, with the source beside it and the row you are working in lit up.
• Edits values by kind — all eleven of them, the four datetime shapes told apart — and converts a value to another kind where it reads as the target.
• Adds, renames, converts and removes keys, tables, inline tables and arrays.
• Edits comments wherever TOML allows one: above a key, beside a value, or at the end of the file.
• Undo and redo, one step per row worked in.
• Saves atomically. The new file replaces the old in one step, so an interruption leaves the old file or the new one, never half of each. A read-only file is refused rather than replaced.

TOML 1.1.0. Every valid case of the toml-test conformance corpus round-trips byte for byte.

Tommy Flyleaf makes no network connection of any kind: no account, no update check, no analytics, no telemetry, no crash reporting. It writes no configuration directory, no history and no list of recent files.

Named for a bookbinder's apprentice. The flyleaf is the blank page inside a book's cover, the one place in a bound book meant for someone to write on later. The book is somebody else's; write on the page provided, and leave the rest as it was bound.

Open source, MIT licensed: github.com/excelano/flyleaf

**What's new in this version**

First release in the Microsoft Store.

The save bullet above said "written beside the original and renamed over it"
when the Microsoft submission was pasted. That is how Windows and Linux do it
and not how the sandboxed Mac build does, so the sentence now says only what is
true everywhere; the Microsoft listing carries the older wording until its next
submission.

**Category**

Developer tools > Utilities

**Search terms** (Microsoft Store: at most 7, each at most 30 characters)

TOML / TOML editor / config editor / configuration file / Cargo.toml / pyproject.toml / developer tools

**Copyright and trademark**

Copyright 2026 David M. Anderson. MIT licensed.

**Additional license terms**

MIT License. Full text: github.com/excelano/flyleaf/blob/main/LICENSE

**Links**

| Field | Value |
| --- | --- |
| Website | https://excelano.com/flyleaf/ |
| Privacy policy | https://excelano.com/legal/#flyleaf |
| Support contact | david.anderson@excelano.com |
| Source | https://github.com/excelano/flyleaf |

**Pricing and availability**

Free, all markets, public.

**Age rating**

No user-generated content, no network access, no data collection, no advertising, no in-app purchases, no violence or mature content.

---

## Notes for certification

The field a reviewer actually reads, and the one that decides whether a
submission comes back. It answers, in order, the four things that would
otherwise be guessed at: how to test an application that opens with nothing in
it, why it does not take the `.toml` default, what the certification kit's one
finding is, and why the no-network claim is checkable rather than asserted.

The empty-window paragraph is not decoration. Launched with no file this
application shows a window with an Open button and a line of text, and a
reviewer who does not know a `.toml` is needed can read that as an application
that does not work.

Microsoft Store limit 2000 characters; this is 1838.

Tommy Flyleaf is a TOML file editor. No account, no sign-in, no test credentials, and no network connection of any kind are needed to test it.

To exercise it: launch it and click Open, or right-click any .toml file and choose Tommy Flyleaf under Open With. Any .toml file will do — a Cargo.toml from a Rust project, a pyproject.toml from a Python one, or a file typed into Notepad with a line such as: title = "hello". On launch with no file the window says so and offers the Open button; that empty state is expected and is not a failure to start.

The application claims .toml only as an Open With entry and never as the default handler. This is deliberate: .toml is a shared extension usually already owned by an editor or an IDE, and taking it would be taking somebody else's association.

The Windows App Certification Kit reports one finding against this package: Blocked executables, naming kernel32.dll!CreateProcessW and the string "cmd.exe". This comes from opening a web link. The About box has two links, to the product page and to the source repository, and the UI framework (egui/eframe) hands a clicked link to the system's default browser through the shell, which is what draws in that API and that string. The application itself spawns no process; the only use of the process API anywhere in the source is reading its own process id, in test code. The report's "rEG" and "dNx" lines are the scanner matching those letters inside a 15 MB binary and refer to no such program.

The application makes no network request. Its import table names nineteen libraries, all of them parts of Windows, and none of them is a networking library: no ws2_32.dll, no winhttp.dll, no wininet.dll. The full privacy statement is at https://excelano.com/legal/#flyleaf and the complete source is at https://github.com/excelano/flyleaf.

---

## Images

| What | Where | Built by |
| --- | --- | --- |
| Screenshot, 1366x768 | `dist/store/01-window.png` | `windows/screenshot.ps1 -File packaging/sample.toml` |
| Mac screenshot, 1440x900 | `dist/store/mac-01-window.png` | `macos/screenshot.sh --app "dist/Tommy Flyleaf.app" --file packaging/sample.toml --out dist/store/mac-01-window.png` |
| Store logo, 1080x1080 | `windows/listing/store-logo-1080.png` | `windows/make-ico` |
| Store logo, 2160x2160 | `windows/listing/store-logo-2160.png` | `windows/make-ico` |

The screenshot is taken from the running application on the sample file, at the
Store's minimum size, by a script that refuses anything else; `screenshot.ps1`
says what it measured about window borders to get there. It is not committed —
it is a photograph of a build, and it is retaken when the interface changes.

---

## The Mac App Store

Most of the above carries over: the name, the description, the links, the
pricing and the age rating are the same product. What differs is Apple's own
fields, and they are here. The Mac screenshot is taken from a development-signed
bundle of the same commit, because a Store-signed one cannot launch off the
Store; `macos/screenshot.sh` refuses any size App Store Connect would.

**Name**

Tommy Flyleaf

**Subtitle** (limit 30; this is 26)

TOML editor, one edit deep

**Promotional text** (limit 170; this is 156)

Open a TOML file as a tree, change one value, and save. Every comment, key and line you did not touch comes back exactly as it was. Nothing leaves your Mac.

**Keywords** (limit 100 characters, one comma-separated string; this is 88)

TOML,editor,config,configuration,Cargo.toml,pyproject.toml,developer,settings,round-trip

**Description**

The Microsoft Store description above, verbatim.

**What's new in this version**

First release on the Mac App Store.

**Category**

Developer Tools

**App Review notes**

Tommy Flyleaf is a TOML file editor. No account, no sign-in, no test credentials, and no network connection of any kind are needed to test it.

To exercise it: launch it and click Open, or drop any .toml file on the Dock icon, or choose Tommy Flyleaf from Open With on one. Any .toml file will do — a Cargo.toml from a Rust project, a pyproject.toml from a Python one, or a file saved from TextEdit with a line such as: title = "hello". On launch with no file the window says so and offers the Open button; that empty state is expected and is not a failure to start.

The application declares the TOML document type and claims it at rank Alternate, not Owner: it is one editor for a format many applications open, and any application that claims .toml at a higher rank keeps double-clicks. On a Mac where nothing else claims the type, macOS will pick it, since there is no other candidate.

The App Sandbox is on with exactly two entitlements: the sandbox itself and read-write access to user-selected files. A save replaces the file the person chose, staged in the replacement directory macOS provides on the file's own volume and swapped in with one call, so it stays inside that grant. There is no network entitlement, and the application makes no network request; the About box has two links, to the product page and to the source repository, which open in the default browser.

The full privacy statement is at https://excelano.com/legal/#flyleaf and the complete source is at https://github.com/excelano/flyleaf.

**Export compliance**

Answered in the bundle: `Info.plist.in` declares `ITSAppUsesNonExemptEncryption` false. The application implements no cryptography.

**Privacy questionnaire**

Data not collected, every category. The statement above is the reason each answer is "no".
