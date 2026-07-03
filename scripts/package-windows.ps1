param(
    [switch]$ZipOnly,
    [ValidateSet("x64", "x86", "arm", "arm64")]
    [string]$Arch = "x64",
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$distRoot = Join-Path $repoRoot "dist"
$targetByArch = @{
    x64   = "x86_64-pc-windows-msvc"
    x86   = "i686-pc-windows-msvc"
    arm   = "aarch64-pc-windows-msvc"
    arm64 = "aarch64-pc-windows-msvc"
}
$targetTriple = $targetByArch[$Arch]
$versionSuffix = if ([string]::IsNullOrWhiteSpace($Version)) { "" } else { "-$Version" }
$packageName = "ripcanvas-windows11-$Arch$versionSuffix"
$packageDir = Join-Path $distRoot $packageName
$zipPath = Join-Path $distRoot "$packageName.zip"
$exePath = Join-Path $repoRoot "target\$targetTriple\release\rocv.exe"

function New-PortableZip {
    Push-Location $repoRoot
    try {
        rustup target add $targetTriple
        cargo build --release --bin rocv --target $targetTriple

        if (Test-Path $packageDir) {
            Remove-Item -LiteralPath $packageDir -Recurse -Force
        }
        New-Item -ItemType Directory -Force -Path $packageDir | Out-Null

        Copy-Item -LiteralPath $exePath -Destination (Join-Path $packageDir "rocv.exe")
        Copy-Item -LiteralPath (Join-Path $repoRoot "assets\icon.ico") -Destination (Join-Path $packageDir "icon.ico")

        @'
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\RipCanvas"
)

$ErrorActionPreference = "Stop"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item -LiteralPath (Join-Path $PSScriptRoot "rocv.exe") -Destination (Join-Path $InstallDir "rocv.exe") -Force

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($userPath -split ";") -notcontains $InstallDir) {
    [Environment]::SetEnvironmentVariable("Path", (($userPath, $InstallDir) -join ";").Trim(";"), "User")
    Write-Host "Installed rocv.exe to $InstallDir and added it to the user PATH."
    Write-Host "Open a new terminal before running rocv from PATH."
} else {
    Write-Host "Installed rocv.exe to $InstallDir."
}
'@ | Set-Content -LiteralPath (Join-Path $packageDir "install.ps1") -Encoding UTF8

        @'
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\RipCanvas"
)

$ErrorActionPreference = "Stop"

if (Test-Path $InstallDir) {
    Remove-Item -LiteralPath $InstallDir -Recurse -Force
}

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$nextPath = (($userPath -split ";") | Where-Object { $_ -and ($_ -ne $InstallDir) }) -join ";"
[Environment]::SetEnvironmentVariable("Path", $nextPath, "User")

Write-Host "Removed RipCanvas from $InstallDir and cleaned the user PATH."
'@ | Set-Content -LiteralPath (Join-Path $packageDir "uninstall.ps1") -Encoding UTF8

        @'
# RipCanvas Windows Portable Package

Run from PowerShell:

```powershell
.\install.ps1
rocv path\to\file.canvas
```

The installer copies `rocv.exe` to `%LOCALAPPDATA%\Programs\RipCanvas` and adds that folder to the user PATH.

Package architecture: PLACEHOLDER_ARCH
'@ | Set-Content -LiteralPath (Join-Path $packageDir "README.md") -Encoding UTF8
        (Get-Content -LiteralPath (Join-Path $packageDir "README.md") -Raw).Replace("PLACEHOLDER_ARCH", $Arch) |
            Set-Content -LiteralPath (Join-Path $packageDir "README.md") -Encoding UTF8

        if (Test-Path $zipPath) {
            Remove-Item -LiteralPath $zipPath -Force
        }
        Compress-Archive -Path (Join-Path $packageDir "*") -DestinationPath $zipPath

        Write-Host "Portable package created:"
        Write-Host "  $packageDir"
        Write-Host "  $zipPath"
    }
    finally {
        Pop-Location
    }
}

Push-Location $repoRoot
try {
    if ($ZipOnly) {
        New-PortableZip
        return
    }

    $packager = Get-Command cargo-packager -ErrorAction SilentlyContinue
    if (-not $packager) {
        $cargoCommands = cargo --list
        $hasCargoSubcommand = $cargoCommands -match "^\s+packager\s"
        if (-not $hasCargoSubcommand) {
            throw "cargo-packager is not installed. Run: cargo install cargo-packager --locked"
        }
    }

    cargo packager --release

    Write-Host "Installer package created under:"
    Write-Host "  $(Join-Path $repoRoot "dist\packager")"
}
finally {
    Pop-Location
}
