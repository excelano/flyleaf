# Remove what install.ps1 put in place, and tell the shell it is gone.
#
# Only what is ours. The `.toml` extension is shared, so its key stays: this
# takes its own ProgID out of the Open With list, clears the default only where
# it is this application's, and leaves the content type, which is a fact about
# the extension. slipcase-desktop's uninstall.ps1 removes its extension whole,
# because that extension is its own; that is the one place the two differ.
#
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)

[CmdletBinding()]
param(
    [string] $Prefix = (Join-Path $env:LOCALAPPDATA 'Programs\Tommy Flyleaf'),
    # Leave the installed executable and icon where they are.
    [switch] $KeepFiles
)

$ErrorActionPreference = 'Stop'

$extension = '.toml'
$progId = 'Excelano.Flyleaf'
$exeName = 'flyleaf.exe'

# A key that is not there is nothing to do; a key that is there and will not go
# is a failure, and the first version of this could not tell the two apart. It
# caught every exception and wrote "nothing at $Path", which is how the
# UserChoice key below survived an uninstall that reported success. Both are
# read back now, so the only thing passed over is the absence.
function Remove-Key {
    param([string] $Path)
    if (-not (Test-OurKey $Path)) { return }
    [Microsoft.Win32.Registry]::CurrentUser.DeleteSubKeyTree($Path, $false)
    if (Test-OurKey $Path) { throw "uninstall.ps1: HKCU\$Path is still there after being deleted" }
}

# The same, for a key that cannot be opened for writing. `DeleteSubKeyTree` and
# `reg delete` both open the key itself with write access before deleting it,
# and Explorer writes a *Deny SetValue* rule on UserChoice so that no
# application can quietly take an extension over. That deny makes the write
# open fail, and then `reg delete` says *Access is denied* while
# `DeleteSubKeyTree` reads the failure as the key being missing and returns
# quietly, which is the half that hid this. Deleting the name from the parent
# needs DELETE on the child and nothing else, which the rule beside the deny
# allows: measured here 2026-09-08, unelevated, on the real key Explorer wrote.
function Remove-Subkey {
    param([string] $Parent, [string] $Name)
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($Parent, $true)
    if (-not $key) { return }
    try { $key.DeleteSubKey($Name, $false) } finally { $key.Close() }
    if (Test-OurKey "$Parent\$Name") {
        throw "uninstall.ps1: HKCU\$Parent\$Name is still there after being deleted"
    }
}

function Test-OurKey {
    param([string] $Path)
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($Path, $false)
    if (-not $key) { return $false }
    $key.Close()
    return $true
}

# A value, or the key's default when $Name is empty, removed only if it holds
# what install.ps1 wrote, so that another application's registration on the
# shared extension is not touched.
function Remove-OurValue {
    param([string] $Path, [string] $Name, [string] $Ours)
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($Path, $true)
    if (-not $key) { return }
    try {
        $held = $key.GetValue($Name, $null)
        if ($null -ne $held -and ($Ours -eq '' -or $held -eq $Ours)) {
            $key.DeleteValue($Name, $false)
        }
    } finally {
        $key.Close()
    }
}

$classes = 'Software\Classes'

Remove-Key "$classes\$progId"
Remove-OurValue "$classes\$extension\OpenWithProgids" $progId ''
Remove-OurValue "$classes\$extension" '' $progId
Remove-Key "$classes\Applications\$exeName"
Remove-Key 'Software\Microsoft\Windows\CurrentVersion\Uninstall\Tommy Flyleaf (script install)'

# The one that is easy to miss. Choosing "always open with" writes a UserChoice
# here, and so does opening a file through the association, measured on this
# machine 2026-09-08; a UserChoice naming a ProgID whose executable is gone
# kills the extension outright, measured in slipcase-desktop. Removed only when
# it names this application, since on a shared extension it may well name
# somebody else's, and that choice is theirs to keep.
#
# The UserChoice key by name through `Remove-Subkey`, and not the
# `FileExts\.toml` tree above it, which is what this deleted first and what
# never worked. It is also the narrower thing to do: that tree holds other
# applications' entries for a shared extension, and none of those are ours.
$exts = "Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\$extension"
$key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey("$exts\UserChoice", $false)
if ($key) {
    $chosen = $key.GetValue('ProgId', $null)
    $key.Close()
    if ($chosen -eq $progId) { Remove-Subkey $exts 'UserChoice' }
}

$shortcut = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Tommy Flyleaf.lnk'
if (Test-Path -LiteralPath $shortcut) { Remove-Item -LiteralPath $shortcut -Force -Confirm:$false }

if (-not $KeepFiles) {
    foreach ($name in $exeName, 'flyleaf.ico') {
        $path = Join-Path $Prefix $name
        if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force -Confirm:$false }
    }
    # Add/Remove Programs points at the copy inside the directory, and a running
    # script cannot delete itself; run from a checkout it is another file and
    # can go with the rest.
    $copy = Join-Path $Prefix 'uninstall.ps1'
    $self = $MyInvocation.MyCommand.Path
    if (Test-Path -LiteralPath $copy) {
        $same = $self -and
            ([System.IO.Path]::GetFullPath($self) -ieq [System.IO.Path]::GetFullPath($copy))
        if ($same) {
            Write-Output "left ${copy} behind: it is the script now running"
        } else {
            Remove-Item -LiteralPath $copy -Force -Confirm:$false
        }
    }
    if ((Test-Path -LiteralPath $Prefix) -and
        -not (Get-ChildItem -LiteralPath $Prefix -Force)) {
        Remove-Item -LiteralPath $Prefix -Force -Confirm:$false
    }
}

Add-Type -Namespace FlyleafUninstall -Name Shell -MemberDefinition @'
[DllImport("shell32.dll", CharSet=CharSet.Unicode)]
public static extern void SHChangeNotify(int eventId, uint flags, System.IntPtr item1, System.IntPtr item2);
'@
[FlyleafUninstall.Shell]::SHChangeNotify(0x08000000, 0, [System.IntPtr]::Zero, [System.IntPtr]::Zero)

Write-Output "removed Tommy Flyleaf from the $extension Open With list, and its Start menu and uninstall entries"
