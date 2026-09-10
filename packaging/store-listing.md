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

**Short description**

Under `## Short description` below, which the Microsoft Store takes verbatim.
**The limit is 500, not the 1,000 the form accepts**: the submission API
refuses anything longer, measured 2026-09-09 and recorded in fenster's
`LIMITS.md`, and it measures the listing already published before it will take
a new package. This text is 145.

**Description**

Under `## Description` below, which both stores take verbatim. The Microsoft
limit is 10,000 and the Mac's is 4,000, so the text is written to the smaller
one and the same words go in both forms.

**What's new in this version**

Under `## Release notes` below, one subsection per version, which is where both
lanes now read it from.

The save bullet in the description said "written beside the original and renamed over it"
when the Microsoft submission was pasted. That is how Windows and Linux do it
and not how the sandboxed Mac build does, so the sentence now says only what is
true everywhere; the Microsoft listing carries the older wording until its next
submission.

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

---

## Short description (Microsoft Store, 500)

Edit a TOML file as a tree, every value by its kind, and save it with the comments, key order and formatting of everything you did not edit kept.

## App features (Microsoft Store, up to 20 bullets of 200 characters)

    Shows the document as a tree, one renderer per TOML type, with the source beside it and the row you are working in lit up.
    Edits values by kind — all eleven of them, the four datetime shapes told apart — and converts a value to another kind where it reads as the target.
    Adds, renames, converts and removes keys, tables, inline tables and arrays, and edits comments wherever TOML allows one. Undo and redo, one step per row worked in.
    A save keeps every comment, the key order, the whitespace and the quoting of everything you did not edit. TOML 1.1.0, and every valid toml-test case round-trips byte for byte.
    Saves atomically. The new file replaces the old in one step, so an interruption leaves the old file or the new one, never half of each. A read-only file is refused rather than replaced.
    Makes no network connection of any kind: no account, no update check, no analytics, no telemetry, no crash reporting. No configuration directory, no history, no list of recent files.

**Every bullet restates something `## Description` already says**, the way
slipcase-desktop's do: the Store shows these as a summary beside the
description, and a feature list making a claim the description does not is a
second listing to keep true. Each one above is the description's bullet of the
same subject, cut to 200 characters. There is no Mac App Store equivalent of
this field.

**Written 2026-09-10, from the description and the changelog.** The 0.2.1
submission typed six features straight into the form, so the Store carries text
this repository never held — the drift the note at the top of this file
forbids. What is above replaces them at the next submission rather than
reproducing them, because nothing here records what they said. Nothing claims
the German drawing added in 0.2.2 or the browser build: the Store serves 0.2.1,
and a feature list is not the place to announce a version nobody can install.

## Subtitle (Mac App Store, 30)

TOML editor, one edit deep

## Promotional text (Mac App Store, 170)

Open a TOML file as a tree, change one value, and save. Every comment, key and line you did not touch comes back exactly as it was. Nothing leaves your Mac.

## Description (both, written to 4,000)

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

## Keywords

**Mac App Store** (100 characters, comma-separated, no spaces after commas):

    TOML,editor,config,configuration,Cargo.toml,pyproject.toml,developer,settings,round-trip

**Microsoft Store** (at most seven terms, each at most 30 characters):

    TOML, TOML editor, config editor, configuration file, Cargo.toml, pyproject.toml, developer tools

## URLs

Both forms ask for the same three, and both lanes take them from here:

| Field | URL |
| --- | --- |
| Privacy policy | https://excelano.com/legal/#flyleaf |
| Support | https://excelano.com/flyleaf/#support |
| Marketing / website | https://excelano.com/flyleaf/ |

The page at `excelano.com/flyleaf/` is the support and marketing URL both, the
way segler's and duckling's are; its *Support* heading is the anchor, read back
off the served page on 2026-09-09. **Support was an email address here until
then**, which App Store Connect will not take in that field: it wants a URL and
the address belongs on the page behind it. The source repository is
`github.com/excelano/flyleaf` and is not one of the three fields either form
asks for; it is in the description, where a reader can follow it.

## Release notes

*What's new in this version* on the Microsoft Store and *What's New* on the Mac
App Store, one version's text each, kept latest first.

**The subsection below carries the Microsoft text.** The Mac App Store has
never served this application, so when that submission comes its text is:
*First release on the Mac App Store.*

### 0.2.3

Tommy Flyleaf draws in German where the desktop asks for German. The window is crisp on a scaled display, and it has its own icon.

### 0.2.1

First release in the Microsoft Store.

## App Review notes

Tommy Flyleaf is a TOML file editor. No account, no sign-in, no test credentials, and no network connection of any kind are needed to test it.

To exercise it: launch it and click Open, or drop any .toml file on the Dock icon, or choose Tommy Flyleaf from Open With on one. Any .toml file will do — a Cargo.toml from a Rust project, a pyproject.toml from a Python one, or a file saved from TextEdit with a line such as: title = "hello". On launch with no file the window says so and offers the Open button; that empty state is expected and is not a failure to start.

The application declares the TOML document type and claims it at rank Alternate, not Owner: it is one editor for a format many applications open, and any application that claims .toml at a higher rank keeps double-clicks. On a Mac where nothing else claims the type, macOS will pick it, since there is no other candidate.

The App Sandbox is on with exactly two entitlements: the sandbox itself and read-write access to user-selected files. A save replaces the file the person chose, staged in the replacement directory macOS provides on the file's own volume and swapped in with one call, so it stays inside that grant. There is no network entitlement, and the application makes no network request; the About box has two links, to the product page and to the source repository, which open in the default browser.

The full privacy statement is at https://excelano.com/legal/#flyleaf and the complete source is at https://github.com/excelano/flyleaf.

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
| Mac screenshot, light, 1440x900 | `dist/store/mac-01-window-light.png` | `macos/screenshot.sh --app "dist-universal/Tommy Flyleaf.app" --file packaging/sample.toml --out dist/store/mac-01-window-light.png`, with the Mac in Light |
| Mac screenshot, dark, 1440x900 | `dist/store/mac-01-window.png` | the same, with the Mac in Dark |
| Store logo, 1080x1080 | `windows/listing/store-logo-1080.png` | `windows/make-ico` |
| Store logo, 2160x2160 | `windows/listing/store-logo-2160.png` | `windows/make-ico` |

The screenshot is taken from the running application on the sample file, at the
Store's minimum size, by a script that refuses anything else; `screenshot.ps1`
says what it measured about window borders to get there. It is not committed —
it is a photograph of a build, and it is retaken when the interface changes.

---

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
