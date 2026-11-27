# Starts Flash Editor, waits until its window is drawn, saves a screenshot of the window and closes it.
# Also prints start-up time, memory and idle CPU (for the progress log).
# Usage:  powershell -ExecutionPolicy Bypass -File scripts\screenshot.ps1 -Out docs\screenshots\compare\name.png [-Release] [-Width 1919 -Height 1035]
# -Steps runs mouse and key steps before the picture is taken, separated by ";" (x, y are inside the window):
#   keys:<SendKeys text>   click:x,y   rclick:x,y (right-click)   dclick:x,y (double-click)   drag:x1,y1,x2,y2   shiftdrag:x1,y1,x2,y2   wait:ms

param(
    [Parameter(Mandatory = $true)][string]$Out,
    [switch]$Release,
    [int]$Width = 0,
    [int]$Height = 0,
    [int]$WaitSeconds = 3,
    [string]$AppArgs = "",
    [string]$Keys = "",
    [string]$Steps = ""
)

$ErrorActionPreference = "Stop"
$root = Join-Path $PSScriptRoot ".."
$exe = Join-Path $root ($(if ($Release) { "target\release\flash-editor.exe" } else { "target\debug\flash-editor.exe" }))
if (-not (Test-Path $exe)) { throw "Build first: $exe not found" }

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System; using System.Text; using System.Runtime.InteropServices;
public static class FeWin {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hh, bool repaint);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte k, byte s, uint f, UIntPtr e);
  public struct RECT { public int L, T, R, B; }
  public static IntPtr Find(uint pid) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((h, l) => { uint p; GetWindowThreadProcessId(h, out p);
      if (p == pid && IsWindowVisible(h)) { var s = new StringBuilder(256); GetWindowText(h, s, 256);
        if (s.ToString().Contains("Flash Editor")) { found = h; return false; } }
      return true; }, IntPtr.Zero);
    return found;
  }
}
"@

$watch = [Diagnostics.Stopwatch]::StartNew()
$p = if ($AppArgs) { Start-Process $exe -ArgumentList $AppArgs -PassThru } else { Start-Process $exe -PassThru }
$h = [IntPtr]::Zero
while ($h -eq [IntPtr]::Zero -and $watch.ElapsedMilliseconds -lt 20000) { Start-Sleep -Milliseconds 50; $h = [FeWin]::Find([uint32]$p.Id) }
if ($h -eq [IntPtr]::Zero) { $p.Kill(); throw "The window did not appear." }
"Window visible after $($watch.ElapsedMilliseconds) ms"
if ($Width -gt 0) { [FeWin]::ShowWindow($h, 9) | Out-Null; [FeWin]::MoveWindow($h, 0, 0, $Width, $Height, $true) | Out-Null }
Start-Sleep -Seconds $WaitSeconds
[FeWin]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Milliseconds 700
if ($Keys) {
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.SendKeys]::SendWait($Keys)
    Start-Sleep -Milliseconds 700
}
$r = New-Object FeWin+RECT; [FeWin]::GetWindowRect($h, [ref]$r) | Out-Null
function Move-To([int]$x, [int]$y) { [FeWin]::SetCursorPos($r.L + $x, $r.T + $y) | Out-Null; Start-Sleep -Milliseconds 60 }
function Invoke-Drag([int[]]$v) {
    Move-To $v[0] $v[1]; [FeWin]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 120
    foreach ($i in 1..12) { Move-To ($v[0] + ($v[2] - $v[0]) * $i / 12) ($v[1] + ($v[3] - $v[1]) * $i / 12) }
    Start-Sleep -Milliseconds 120; [FeWin]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 300
}
if ($Steps) {
    Add-Type -AssemblyName System.Windows.Forms
    foreach ($step in $Steps.Split(";")) {
        $kind, $value = $step.Split(":", 2)
        $v = if ($kind -ne "keys") { [int[]]($value.Split(",")) } else { $null }
        switch ($kind) {
            "keys" { [System.Windows.Forms.SendKeys]::SendWait($value); Start-Sleep -Milliseconds 400 }
            "dclick" { Move-To $v[0] $v[1]; [FeWin]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); [FeWin]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 60; [FeWin]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); [FeWin]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 500 }
            "rclick" { Move-To $v[0] $v[1]; [FeWin]::mouse_event(8, 0, 0, 0, [UIntPtr]::Zero); [FeWin]::mouse_event(16, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 500 }
            "click" { Move-To $v[0] $v[1]; [FeWin]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); [FeWin]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 500 }
            "drag" { Invoke-Drag $v }
            "shiftdrag" { [FeWin]::keybd_event(0x10, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 100; Invoke-Drag $v; [FeWin]::keybd_event(0x10, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 300 }
            "wait" { Start-Sleep -Milliseconds $v[0] }
        }
    }
    Move-To 5 5
    Start-Sleep -Milliseconds 500
}
$bmp = New-Object System.Drawing.Bitmap ($r.R - $r.L), ($r.B - $r.T)
[System.Drawing.Graphics]::FromImage($bmp).CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
$full = [IO.Path]::GetFullPath((Join-Path (Get-Location) $Out))
New-Item -ItemType Directory -Force (Split-Path $full) | Out-Null
$bmp.Save($full)
"Saved $($bmp.Width) x $($bmp.Height) to $full"
$p.Refresh(); $c1 = $p.TotalProcessorTime; Start-Sleep -Seconds 3; $p.Refresh(); $c2 = $p.TotalProcessorTime
"Memory: $([math]::Round($p.WorkingSet64 / 1MB, 1)) MB working set; idle CPU over 3 s: $([math]::Round(($c2 - $c1).TotalMilliseconds)) ms"
$p.CloseMainWindow() | Out-Null; Start-Sleep -Milliseconds 800; if (-not $p.HasExited) { $p.Kill() }
