# Builds the Flash Editor installer into installer\output:  FlashEditor-Setup-<version>.exe
# (One edition only: the AI models are not inside; setup offers to download them, or the AI tools do on first use.)
#
# Needs: Rust (MSVC) and Inno Setup 6 (winget install JRSoftware.InnoSetup).
# The three AI runtime DLLs are taken from a folder (default: the C# build's, like scripts\get-ai-runtime.ps1).
#
# Usage (from the project folder):
#   powershell -ExecutionPolicy Bypass -File installer\build-installer.ps1
#   powershell -ExecutionPolicy Bypass -File installer\build-installer.ps1 -Version 1.0.0 -RuntimeFrom D:\path\to\dlls -TestInstall

param(
    [string]$Version = "",
    [string]$RuntimeFrom = "D:\AiBuild\Flash Editor\installer\build\app",
    [switch]$TestInstall   # no administrator needed: for trying the installer into a folder of your own
)

$ErrorActionPreference = "Stop"
$installer = $PSScriptRoot
$root = Split-Path $installer -Parent
$build = Join-Path $installer "build\app"

if (-not $Version) {
    $Version = (Select-String -Path (Join-Path $root "Cargo.toml") -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
}

# --- Inno Setup compiler -------------------------------------------------------
$iscc = @(
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $iscc) { throw "Inno Setup 6 not found. Install it with: winget install JRSoftware.InnoSetup" }

# --- 1. The release program ----------------------------------------------------
Write-Host "Building Flash Editor $Version..." -ForegroundColor Cyan
Push-Location $root
try { cargo build -p fe-app --release; if ($LASTEXITCODE -ne 0) { throw "cargo build failed" } } finally { Pop-Location }
if (Test-Path $build) { Remove-Item $build -Recurse -Force }
New-Item -ItemType Directory -Force $build | Out-Null
Copy-Item (Join-Path $root "target\release\flash-editor.exe") $build
foreach ($dll in "onnxruntime.dll", "onnxruntime_providers_shared.dll", "DirectML.dll") {
    $from = Join-Path $RuntimeFrom $dll
    if (-not (Test-Path $from)) { throw "AI runtime file missing: $from (pass -RuntimeFrom)" }
    Copy-Item $from $build
}

# --- 2. The installer ----------------------------------------------------------
New-Item -ItemType Directory -Force "$installer\output" | Out-Null
$defines = @("/DAppVersion=$Version")
if ($TestInstall) { $defines += "/DTestInstall" }
Write-Host "Building the installer..." -ForegroundColor Cyan
& $iscc /Q @defines "$installer\FlashEditor.iss"
if ($LASTEXITCODE -ne 0) { throw "Inno Setup failed" }

Get-ChildItem "$installer\output" -Filter *.exe |
    Select-Object Name, @{ n = "Size (MB)"; e = { [math]::Round($_.Length / 1MB, 1) } } | Format-Table -AutoSize
Write-Host "Done. The installer is in $installer\output" -ForegroundColor Green
