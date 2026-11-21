# Puts the AI runtime (the three ONNX Runtime / DirectML DLLs) next to the program, and optionally the models too,
# so the AI features work without downloading anything. The program finds them in its own folder.
#
#   powershell -ExecutionPolicy Bypass -File scripts\get-ai-runtime.ps1                 # debug + release folders
#   powershell -ExecutionPolicy Bypass -File scripts\get-ai-runtime.ps1 -WithModels     # also copy the models
#   powershell -ExecutionPolicy Bypass -File scripts\get-ai-runtime.ps1 -Source "D:\path\to\app" -Models "D:\path\to\models"
#
# Without -Source it uses the build folder of the C# version (read only, never changed).

param(
  [string]$Source = "D:\AiBuild\Flash Editor\installer\build\app",
  [string]$Models = "D:\AiBuild\Flash Editor\installer\build\models",
  [string[]]$Targets = @("target\debug", "target\release"),
  [switch]$WithModels
)

$root = Split-Path -Parent $PSScriptRoot
$dlls = "onnxruntime.dll", "onnxruntime_providers_shared.dll", "DirectML.dll"

foreach ($t in $Targets) {
  $dest = Join-Path $root $t
  if (-not (Test-Path $dest)) { continue }
  foreach ($d in $dlls) {
    $from = Join-Path $Source $d
    if (Test-Path $from) { Copy-Item $from $dest -Force } else { Write-Warning "Missing $from" }
  }
  if ($WithModels -and (Test-Path $Models)) {
    Copy-Item $Models (Join-Path $dest "models") -Recurse -Force
  }
  Write-Host "AI runtime copied to $dest"
}
