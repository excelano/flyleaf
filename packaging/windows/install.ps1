# Install the Windows integration: the `.toml` association, the icon, and the
# entry that opens a file. Optionally the executable alongside them.
#
# Per-user, under HKCU and %LOCALAPPDATA%, which is the counterpart of the
# Linux script's default of ~/.local: no administrator, and nothing written
# that another account can see. Adapted from slipcase-desktop's install.ps1,
# where every registry choice below was measured; what differs here is that
# `.toml` is a shared extension this application does not own, so by default
# it is added to the Open With list and not made the handler. `-Default` makes
# it the handler, the way `packaging/linux/install.sh --default` does.
#
# Author: David M. Anderson
# Built with AI assistance (Claude, Anthropic)

[CmdletBinding()]
param(
    # Where to install. The default is the per-user location Windows names for
    # applications that do not go through an installer service.
    [string] $Prefix = (Join-Path $env:LOCALAPPDATA 'Programs\Tommy Flyleaf'),
    # The executable to install. With neither this nor -NoBinary, a built one
    # is looked for.
    [string] $Binary,
    # Install the integration only.
    [switch] $NoBinary,
    # Make this application what a double-clicked .toml opens with. Off by
    # default: a .toml is usually somebody else's file and often already
    # somebody else's association, and an install script does not take that
    # decision on its own.
    [switch] $Default
)

$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path

# The extension is TOML's own and the content type is the one IANA registered
# for it in 2024; neither was chosen here.
$extension = '.toml'
$contentType = 'application/toml'

# Chosen here. `Vendor.Component` is the shape Windows documents for a ProgID.
$progId = 'Excelano.Flyleaf'
$typeName = 'TOML document'
$appName = 'Tommy Flyleaf'
$exeName = 'flyleaf.exe'
$iconName = 'flyleaf.ico'
$description = 'Edit a TOML file as a tree and keep the rest of the file as it was'

# --- writing to the registry ------------------------------------------------

# The .NET API rather than PowerShell's registry provider: the provider reads
# a forward slash in a key name as a path separator, which slipcase-desktop
# measured on its media-type key, and the same API is used here so that the
# two scripts stay one script. An empty $Name is the key's default value.
function Set-RegistryValue {
    param([string] $Path, [string] $Name, $Value, [string] $Kind = 'String')
    $key = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey($Path)
    try {
        $key.SetValue($Name, $Value, [Microsoft.Win32.RegistryValueKind] $Kind)
    } finally {
        $key.Close()
    }
}

# --- the executable ---------------------------------------------------------

# Cargo is asked where its target directory is rather than guessed at, because
# `[build] target-dir` in a Cargo configuration file moves it and no
# environment variable then says so.
function Find-Binary {
    $targetDir = $null
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Push-Location (Join-Path $here '..\..')
        try {
            $meta = cargo metadata --format-version 1 --no-deps 2>$null | ConvertFrom-Json
            if ($meta) { $targetDir = $meta.target_directory }
        } catch { }
        finally { Pop-Location }
    }
    if (-not $targetDir) { $targetDir = Join-Path $here '..\..\target' }

    foreach ($built in 'release', 'debug') {
        $candidate = Join-Path $targetDir "$built\$exeName"
        if (Test-Path -LiteralPath $candidate) { return (Resolve-Path -LiteralPath $candidate).Path }
    }
    return $null
}

$foundBinary = $null
if ($NoBinary) {
    # Nothing to find.
} elseif ($Binary) {
    if (-not (Test-Path -LiteralPath $Binary)) { throw "install.ps1: $Binary is not there" }
    $foundBinary = (Resolve-Path -LiteralPath $Binary).Path
} else {
    $foundBinary = Find-Binary
}

# --- the files --------------------------------------------------------------

New-Item -ItemType Directory -Force -Path $Prefix | Out-Null

$iconSource = Join-Path $here $iconName
if (-not (Test-Path -LiteralPath $iconSource)) {
    throw "install.ps1: $iconName is not beside this script; run make-ico first"
}
$installedIcon = Join-Path $Prefix $iconName
Copy-Item -LiteralPath $iconSource -Destination $installedIcon -Force

# The uninstaller is copied in rather than run from the repository, because the
# Add/Remove Programs entry below points at it and a checkout is not something
# that has to still be there a year later.
Copy-Item -LiteralPath (Join-Path $here 'uninstall.ps1') `
          -Destination (Join-Path $Prefix 'uninstall.ps1') -Force

$installedExe = Join-Path $Prefix $exeName
if ($foundBinary) {
    # An upgrade over a running copy is the one failure here a person meets in
    # the ordinary course of things, and Windows will not let a running
    # executable be overwritten. The script stops before the registry stage,
    # so nothing is left half-registered.
    try {
        Copy-Item -LiteralPath $foundBinary -Destination $installedExe -Force
    } catch [System.IO.IOException] {
        $running = Get-Process -Name ([System.IO.Path]::GetFileNameWithoutExtension($exeName)) `
                               -ErrorAction SilentlyContinue |
                   Where-Object { $_.Path -eq $installedExe }
        if ($running) {
            throw "install.ps1: Tommy Flyleaf is running from $installedExe, so it cannot be replaced. " +
                  "Close it and run this again. Nothing has been changed."
        }
        throw
    }
    Write-Output "installed $installedExe from $foundBinary"
} elseif (-not (Test-Path -LiteralPath $installedExe)) {
    Write-Warning "no executable installed; the association will point at $installedExe, which is not there yet"
}


# --- the registry -----------------------------------------------------------

$classes = 'Software\Classes'

# The type as this application describes it. `FriendlyTypeName` is a plain
# string rather than a resource reference, for the reason slipcase-desktop
# records: a reference needs SHLoadIndirectString to read back.
Set-RegistryValue "$classes\$progId" '' $typeName
Set-RegistryValue "$classes\$progId" 'FriendlyTypeName' $typeName
Set-RegistryValue "$classes\$progId\DefaultIcon" '' "$installedIcon,0"
Set-RegistryValue "$classes\$progId\shell\open\command" '' "`"$installedExe`" `"%1`""

# The application behind the type. `ApplicationName` is the first place the
# shell looks for a name a person recognises.
Set-RegistryValue "$classes\$progId\Application" 'ApplicationName' $appName
Set-RegistryValue "$classes\$progId\Application" 'ApplicationCompany' 'Excelano'
Set-RegistryValue "$classes\$progId\Application" 'ApplicationDescription' $description
Set-RegistryValue "$classes\$progId\Application" 'ApplicationIcon' "$installedIcon,0"

# The extension. Only the Open With list by default, since the extension is
# shared: the key's default value is what a double-click follows, and it is
# written only when asked. The content type is a fact about the extension and
# not about the handler, so it is written either way and left alone on
# uninstall.
Set-RegistryValue "$classes\$extension\OpenWithProgids" $progId ''
Set-RegistryValue "$classes\$extension" 'Content Type' $contentType
if ($Default) {
    Set-RegistryValue "$classes\$extension" '' $progId
}

# The Open With list, so a person can reach this application from a file it
# was not registered for, and so the shell has a name for the executable.
$applications = "$classes\Applications\$exeName"
Set-RegistryValue $applications 'FriendlyAppName' $appName
Set-RegistryValue "$applications\shell\open\command" '' "`"$installedExe`" `"%1`""
Set-RegistryValue "$applications\SupportedTypes" $extension ''

# --- the Start menu ---------------------------------------------------------

# The counterpart of the `.desktop` entry. No AppUserModelID, for the reason
# slipcase-desktop's README records: with neither the shortcut nor the process
# declaring one, Windows derives both from the executable's path and they
# agree.
$startMenu = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$shortcut = Join-Path $startMenu 'Tommy Flyleaf.lnk'
if (Test-Path -LiteralPath $installedExe) {
    $shell = New-Object -ComObject WScript.Shell
    $link = $shell.CreateShortcut($shortcut)
    $link.TargetPath = $installedExe
    $link.WorkingDirectory = $Prefix
    $link.IconLocation = "$installedIcon,0"
    $link.Description = $description
    $link.Save()
}

# --- Add/Remove Programs ----------------------------------------------------

# Named apart from the Store package, so that a person with both can tell in
# Add/Remove Programs which one this is and remove it before installing from
# the Store; the two registered at once put up Windows' picker on every
# double-click, measured in slipcase-desktop.
$version = '0.0.0'
$cargoToml = Join-Path $here '..\..\Cargo.toml'
if (Test-Path -LiteralPath $cargoToml) {
    $line = Select-String -LiteralPath $cargoToml -Pattern '^version = "([^"]+)"' | Select-Object -First 1
    if ($line) { $version = $line.Matches[0].Groups[1].Value }
}

$uninstallKey = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\Tommy Flyleaf (script install)'
$uninstallCommand = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$(Join-Path $Prefix 'uninstall.ps1')`""
Set-RegistryValue $uninstallKey 'DisplayName' 'Tommy Flyleaf (script install)'
Set-RegistryValue $uninstallKey 'DisplayVersion' $version
Set-RegistryValue $uninstallKey 'Publisher' 'Excelano'
Set-RegistryValue $uninstallKey 'DisplayIcon' "$installedIcon,0"
Set-RegistryValue $uninstallKey 'InstallLocation' $Prefix
Set-RegistryValue $uninstallKey 'UninstallString' $uninstallCommand
Set-RegistryValue $uninstallKey 'QuietUninstallString' $uninstallCommand
Set-RegistryValue $uninstallKey 'NoModify' 1 'DWord'
Set-RegistryValue $uninstallKey 'NoRepair' 1 'DWord'

# --- tell the shell ---------------------------------------------------------

# Without this the icon and the type description appear at the next logon
# rather than now, which reads as the association not having worked.
Add-Type -Namespace Flyleaf -Name Shell -MemberDefinition @'
[DllImport("shell32.dll", CharSet=CharSet.Unicode)]
public static extern void SHChangeNotify(int eventId, uint flags, System.IntPtr item1, System.IntPtr item2);
'@
[Flyleaf.Shell]::SHChangeNotify(0x08000000, 0, [System.IntPtr]::Zero, [System.IntPtr]::Zero)

Write-Output ""
if ($Default) {
    Write-Output "registered $progId as what opens $extension, under $Prefix"
} else {
    Write-Output "registered $progId in the Open With list for $extension, under $Prefix (-Default makes it the handler)"
}
Write-Output ""
Write-Output "check it with:"
# Not `assoc` and `ftype`, which read the machine-wide half of the class root
# only and report a per-user install as no association at all.
Write-Output "  reg query `"HKCU\Software\Classes\$extension`" /s"
Write-Output "and by right-clicking a $extension file: Tommy Flyleaf is under Open With."
