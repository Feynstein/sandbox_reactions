# PowerShell launcher: starts the game, waits for Enter, stops it (the lead asked for a .ps1 beside start.bat).
# Run: powershell -ExecutionPolicy Bypass -File start.ps1   (optional -Service <launch.json service>)
param([string]$Service = "game")
Set-Location -LiteralPath $PSScriptRoot
$asked = @()
if ($Service -eq "game") { $asked = @("--asked") }   # running this script is the ask (R4)
py -3.12 tools/pb/launch.py start $Service --task launcher @asked
Read-Host "Sandbox Reactions is running. Press Enter to close it" | Out-Null
py -3.12 tools/pb/launch.py stop --task launcher
