# Runs every code check: formatting, lints (warnings count as errors) and tests.
# Usage (from anywhere):  powershell -ExecutionPolicy Bypass -File scripts\check.ps1

$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

function Invoke-Step($name, [scriptblock]$command) {
    Write-Host "== $name" -ForegroundColor Cyan
    & $command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $name" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

Invoke-Step "Format"  { cargo fmt --all -- --check }
Invoke-Step "Clippy"  { cargo clippy --workspace --all-targets -- -D warnings }
Invoke-Step "Tests"   { cargo test --workspace }

Write-Host "All checks passed." -ForegroundColor Green
