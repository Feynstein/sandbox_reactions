# PowerShell launcher: starts the game, waits for Enter, stops it (the lead asked for a .ps1 beside start.bat).
# Run: powershell -ExecutionPolicy Bypass -File start.ps1   (optional -Service <launch.json service>)
param([string]$Service = "game")
Set-Location -LiteralPath $PSScriptRoot
$asked = @()
if ($Service -eq "game") { $asked = @("--asked") }   # running this script is the ask (R4)
py -3.12 tools/pb/launch.py start $Service --task launcher @asked
if ($LASTEXITCODE -ne 0) {   # a start that failed is not a game running: say so, name its log, ask for Enter only to close this window (M0-D129)
    Write-Host "Sandbox Reactions did NOT start (launch.py start exit $LASTEXITCODE). The lines above say why; its log: milestones/m0/logs/launcher.launch.$Service.log"
    Read-Host "Press Enter to close this window" | Out-Null
    exit 1
}
Read-Host "Sandbox Reactions is running. Press Enter to close it" | Out-Null
py -3.12 tools/pb/launch.py stop $Service --task launcher
