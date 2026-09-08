# Photograph the application's own window at a size the Microsoft Store accepts.
#
# Screenshots were once listed as something no script could do, and that was an
# assumption rather than a measurement. What a script cannot do is
# decide which file to open and whether the result is a good advertisement.
# What it can do is every mechanical part: size the window so the visible frame
# is exactly the size asked for, bring it to the front, capture it, and refuse if
# what came back is the wrong size.
#
#   powershell -ExecutionPolicy Bypass -File packaging\windows\screenshot.ps1 `
#       -File C:\path\to\sample.toml -Out C:\path\to\01-window.png
#
# TWO THINGS MEASURED RATHER THAN ASSUMED, BOTH OF THEM PIXELS
#
# `SetWindowPos` sizes the *window rect*, which on Windows 10 carries an
# invisible resize border outside the visible frame: asking for 1366x768 gave a
# frame of 1352x761, which is under the Store's minimum. The visible frame is
# `DWMWA_EXTENDED_FRAME_BOUNDS`, and the difference measured here is 14 by 7.
#
# And that frame's top edge is one pixel above what is actually drawn, so a
# capture at exactly the frame rect picks up a sliver of whatever is behind. It
# arrived as a strip of console text across the top of the first two attempts.
# The capture is two rows taller than needed and the top two are cropped.
#
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)

[CmdletBinding()]
param(
    # The file to open. `packaging/sample.toml` is what the marketing page and
    # the demo show, and the store listing wants the same one.
    [Parameter(Mandatory = $true)][string] $File,
    [Parameter(Mandatory = $true)][string] $Out,
    # The Store's minimum for a desktop screenshot, and the default because a
    # window this size looks like a window rather than like an advertisement.
    [int] $Width = 1366,
    [int] $Height = 768,
    # Where to put the window. Anywhere it fits entirely on screen.
    [int] $X = 200,
    [int] $Y = 100
)

$ErrorActionPreference = 'Stop'

function Refuse([string] $message) { Write-Error "screenshot.ps1: $message" }

if (-not (Test-Path $File)) { Refuse "no file at $File" }
$File = (Resolve-Path $File).Path

Add-Type -AssemblyName System.Drawing
Add-Type -Namespace Shot -Name Win -MemberDefinition @'
[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
[DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
[DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int attr, out RECT r, int size);
public struct RECT { public int Left, Top, Right, Bottom; }
'@

# The association opens it, which is the packaged application where one is
# installed - the build a person actually gets. Anything already running is
# stopped first, so the window being photographed is the one holding this
# file and not a previous one. A `.toml` is often somebody else's by default,
# so `Start-Process` reaches this application only where `install.ps1
# -Default` made it the handler, which is what a machine used for screenshots
# should have.
Get-Process flyleaf -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
Start-Process $File

# Waited for rather than slept through. This was a flat six seconds, and the
# first run on the Windows machine 2026-09-08 refused with the message below
# against a process that was starting perfectly well: a cold 15 MB binary off a
# disk that has never read it takes longer than that to put a window up, and the
# same run a minute later took under two seconds. A fixed sleep that is long
# enough for a cold start is time paid on every warm one, and the message it
# fails with sends the reader to the association, which was not the fault.
$deadline = (Get-Date).AddSeconds(60)
do {
    Start-Sleep -Milliseconds 500
    $app = Get-Process flyleaf -ErrorAction SilentlyContinue | Select-Object -First 1
} while ((-not $app -or $app.MainWindowHandle -eq [IntPtr]::Zero) -and (Get-Date) -lt $deadline)

if (-not $app) {
    Refuse "nothing opened $File - is this application the .toml handler here?"
}
if ($app.MainWindowHandle -eq [IntPtr]::Zero) {
    Refuse "$($app.Path) started and put no window up within 60 seconds"
}
$handle = $app.MainWindowHandle

# The border measured on this platform, and the two spare rows for the crop.
$BORDER_W = 14
$BORDER_H = 7
$TOPMOST = [IntPtr](-1)
$NOTOPMOST = [IntPtr](-2)

[void][Shot.Win]::SetWindowPos(
    $handle, $TOPMOST, $X, $Y, $Width + $BORDER_W, $Height + $BORDER_H + 2, 0)
[void][Shot.Win]::SetForegroundWindow($handle)

# The pointer goes somewhere the window is not, because egui draws hover state
# and the capture keeps it. Measured 2026-08-29: a retake landed with the mouse
# resting over an "add a key" field, which came out highlighted and focus-ringed
# in a picture meant to show the application at rest, and with the scroll bar
# drawn because the pointer was inside the scroll area. Neither is wrong and
# both are noise a shopper reads as an interface doing something.
#
# Bottom right of the virtual screen rather than a constant: the window is
# placed near the top left, and a fixed 1900x1200 is off-screen on a smaller
# display, where Windows clamps it to an edge the window might occupy.
Add-Type -AssemblyName System.Windows.Forms
$away = [System.Windows.Forms.SystemInformation]::VirtualScreen
[System.Windows.Forms.Cursor]::Position =
    New-Object System.Drawing.Point(($away.Right - 2), ($away.Bottom - 2))

# Long enough for the window to settle and repaint at its new size. egui draws
# on demand, and a capture taken during the resize catches a half-laid-out frame.
Start-Sleep -Seconds 3

$rect = New-Object Shot.Win+RECT
[void][Shot.Win]::DwmGetWindowAttribute($handle, 9, [ref]$rect, 16)
$frameW = $rect.Right - $rect.Left
$frameH = $rect.Bottom - $rect.Top
if ($frameW -lt $Width -or $frameH -lt $Height + 2) {
    Refuse "the visible frame came back ${frameW}x${frameH}, which cannot yield ${Width}x${Height} - the resize border is not what this script measured"
}

$full = New-Object System.Drawing.Bitmap($frameW, $frameH)
$graphics = [System.Drawing.Graphics]::FromImage($full)
$graphics.CopyFromScreen(
    $rect.Left, $rect.Top, 0, 0, (New-Object System.Drawing.Size($frameW, $frameH)))
$graphics.Dispose()

$shot = $full.Clone(
    (New-Object System.Drawing.Rectangle(0, 2, $Width, $Height)), $full.PixelFormat)
$shot.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$shot.Dispose()
$full.Dispose()

[void][Shot.Win]::SetWindowPos($handle, $NOTOPMOST, 0, 0, 0, 0, 0x0003)

# Read back rather than trusted. A screenshot of the wrong size is refused at
# upload, and this is the one property of it a machine can check.
$written = [System.Drawing.Image]::FromFile((Resolve-Path $Out).Path)
$got = "$($written.Width)x$($written.Height)"
$written.Dispose()
if ($got -ne "${Width}x${Height}") {
    Refuse "wrote $got and the Store was asked for ${Width}x${Height}"
}
Write-Host "wrote $Out - $got, from $File"
Write-Host 'look at it before it goes anywhere: a correct size is not a good screenshot'
