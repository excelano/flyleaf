# Submission notes

What a store submission needs from a person and no file supplies. The listing
text, and each store's notes for its reviewer, are `store-listing.toml`, and
what a release changed is `release-notes.toml`; the release apps push both. A
field edited here reaches nobody.

## Capability justification

Partner Center asks once, when `runFullTrust` is first declared, and keeps the
answer against the product. 500 characters, newlines counted.

> Tommy Flyleaf is a full-trust Win32 desktop application packaged as MSIX. It
> needs this capability to run at all. It opens the TOML file it was launched
> with, or one chosen through Open, and a save writes back only that file. It
> makes no network connection of any kind, needs no broad filesystem access,
> and uses no device.

## The answers a form asks

    Category                  Developer tools (both stores)
    Copyright and trademark   Excelano LLC
    Additional license terms  blank; the application is MIT and the
                              repository carries the license
    Privacy policy            https://excelano.com/legal/#flyleaf
    Age rating                no objectionable content of any kind
    App Privacy (Apple)       Data Not Collected
    Export compliance         answered in the bundle: Info.plist.in declares
                              ITSAppUsesNonExemptEncryption false
    Price                     free

## Screenshots

Taken by `shots-winstore` and `shots-macstore` from the recipes in
`windows/shots.ps1` and `macos/shots.sh`, over `sample.toml` and its German
counterpart `sample.de.toml`, each staged under the name a person would give
it because the path shows in the frame. The Mac set is an edit under way in
every frame, in light and then dark.
