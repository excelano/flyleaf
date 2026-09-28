# Submission notes

What a store submission needs from a person and no file supplies: the answers
a form asks that no build can give, and the reasoning behind the screenshots.

The listing text itself is not here. It is `store-listing.toml` beside this,
which `ship` checks before the tag and pushes to both stores on every release,
and what a release tells them changed is `release-notes.toml`. A field edited
in this file would reach nobody. Apple's Notes for Review and Microsoft's
Notes for certification are there too, as `apple-review-notes` and
`microsoft-review-notes`, which `ship` pushes to `appStoreReviewDetail` and
`NotesForCertification` on every submission.

**No identity values.** The package name, publisher, family name and store id
Partner Center assigns are in `windows/identity.psd1`, which is not committed,
for the reason that file's committed example gives. The filled-in sheet for a
submission, identity and all, goes in `windows/SUBMITTING.local.md`, which
`.gitignore` reserves.

## Capability justification

The package declares `runFullTrust`, and Partner Center's justification field
for it is asked once when the capability is first declared on the product
rather than on every resubmission - it is not part of the submission document
`ship` reads and writes, and this repo's own resubmissions have gone to
certification since without one being sent. 500-character limit, which counts
newlines.

> Tommy Flyleaf is a full-trust Win32 desktop application packaged as MSIX. It
> needs this capability to run at all. It opens the TOML file it was launched
> with, or one chosen through Open, and a save writes back only that file. It
> makes no network connection of any kind, needs no broad filesystem access,
> and uses no device.

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

## Notes for certification

Now `microsoft-review-notes` in `store-listing.toml`, which `ship` pushes to
`NotesForCertification` on every submission. It answers, in order, the four
things a certification reader would otherwise have to guess at: how to test an
application that opens with nothing in it, why it does not take the `.toml`
default, what the certification kit's one finding is, and why the no-network
claim is checkable rather than asserted. Microsoft Store limit 2000
characters; the current text is 1964.

**It names GitHub and not `excelano.com/flyleaf/samples/`**, although the page
exists and says more, because the repository was serving the files the hour
the answer was needed and a site deploy is a step in someone's day. Naming an
address that 404s in the field a reviewer clicks is how one rejection becomes
two. The page is the better address once it is live, and the notes can take it
at the next submission.

The empty-window paragraph is not decoration. Launched with no file this
application shows a window with an Open button and a line of text, and a
reviewer who does not know a `.toml` is needed can read that as an application
that does not work. Nor is the sample-files line. Apple's reviewer had these
notes in their Mac wording, empty-window paragraph and all, and came back on
2026-09-10 asking for files anyway: telling a reviewer that any `.toml` will do
asks them to make one, and an address hands them four.

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
