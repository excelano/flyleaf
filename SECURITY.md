# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately, using the **Report a vulnerability** button on the Security tab of this repository. If that button is not there, or you would rather not go through GitHub, email david.anderson@excelano.com instead. I aim to respond within seven days.

Please do not open public issues for security problems.

## Supported versions

The latest release receives security fixes. Older versions are not supported. A fix ships in a new release; update the way you installed.

## What Tommy Flyleaf can access

Tommy Flyleaf runs locally. It reads the TOML file it was launched with or the one chosen through Open, and writes only where a save or a Save as puts it. A save writes the new text to a temporary file on the same volume and renames it over the original, so an interruption leaves one file or the other whole; on macOS, where the App Sandbox grants the chosen file and not its folder, the temporary file is staged in the directory macOS provides for replacement and swapped in by macOS. A read-only file is refused rather than replaced. Parsing is `toml_edit`'s, and nothing in a document is executed.

It makes no network connection. The About box holds two links, and clicking one hands the address to the system's browser. On Linux the window talks to the display server and the desktop's file-dialog and settings portals over local sockets. On macOS the sandbox entitlements are the sandbox itself and read-write access to files the person selects, with no network entitlement. In a browser, a file arrives from the browser's picker into the page's memory and a save is a download; the page has no server side.

## What Tommy Flyleaf stores

Nothing but the files you save. There is no configuration directory, no recent-files list, no cache, no telemetry and no remote logging. On macOS the system's open and save panels keep their own preferences (their size and the last folder shown) in the sandbox container at `~/Library/Containers/com.excelano.flyleaf`. The web build uses no cookies and no browser storage.
