# Starts Flash Editor and samples its memory and CPU while it runs (for the Phase 16 numbers in the progress log).
# Usage:  powershell -ExecutionPolicy Bypass -File scripts\measure.ps1 [-Args "--bench"] [-Seconds 12] [-Release]
# Prints the start-up time, the peak and final working set / private memory and the CPU time used.

param(
    [string]$AppArgs = "",
    [int]$Seconds = 12,
    [switch]$Release
)

$root = Join-Path $PSScriptRoot ".."
$exe = Join-Path $root ($(if ($Release) { "target\release\flash-editor.exe" } else { "target\debug\flash-editor.exe" }))
if (-not (Test-Path $exe)) { throw "Build first: $exe not found" }

$start = Get-Date
$p = if ($AppArgs) { Start-Process $exe -ArgumentList $AppArgs -PassThru } else { Start-Process $exe -PassThru }
$peakWs = 0; $peakPriv = 0; $first = $null
for ($i = 0; $i -lt $Seconds * 10; $i++) {
    Start-Sleep -Milliseconds 100
    if ($p.HasExited) { break }
    $p.Refresh()
    if (-not $first -and $p.MainWindowHandle -ne 0) { $first = [math]::Round(((Get-Date) - $start).TotalMilliseconds) }
    $peakWs = [math]::Max($peakWs, $p.WorkingSet64)
    $peakPriv = [math]::Max($peakPriv, $p.PrivateMemorySize64)
}
if (-not $p.HasExited) {
    $p.Refresh()
    $ws = $p.WorkingSet64; $priv = $p.PrivateMemorySize64; $cpu = $p.TotalProcessorTime.TotalSeconds
    $c1 = $p.TotalProcessorTime; Start-Sleep -Seconds 3; $p.Refresh(); $idle = ($p.TotalProcessorTime - $c1).TotalMilliseconds
    Stop-Process $p -Force
    "Window after ${first} ms; working set peak $([math]::Round($peakWs / 1MB)) MB, now $([math]::Round($ws / 1MB)) MB; private peak $([math]::Round($peakPriv / 1MB)) MB, now $([math]::Round($priv / 1MB)) MB; CPU used $([math]::Round($cpu, 1)) s; idle CPU over 3 s: $([math]::Round($idle)) ms"
} else {
    "Exited after $([math]::Round(((Get-Date) - $start).TotalSeconds, 1)) s; working set peak $([math]::Round($peakWs / 1MB)) MB; private peak $([math]::Round($peakPriv / 1MB)) MB"
}
