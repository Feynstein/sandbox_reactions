@echo off
rem Double-click launcher for Windows: starts the game, waits for Enter, stops it (PLAYBOOK section 8, contract section 3.6 [M0-TJ3]).
rem Optional first argument: another launch.json service (a check runs "start.bat game-offscreen").
cd /d "%~dp0"
set "SERVICE=%~1"
if "%SERVICE%"=="" set "SERVICE=game"
set "ASKED="
if "%SERVICE%"=="game" set "ASKED=--asked"
py -3.12 tools\pb\launch.py start %SERVICE% --task launcher %ASKED%
if errorlevel 1 goto failed
echo Sandbox Reactions is running. Press Enter to close it.
set /p "_="
py -3.12 tools\pb\launch.py stop %SERVICE% --task launcher
goto :eof
:failed
rem A start that failed is not a game running (M0-D129): say so, name its log, ask for Enter only to close this window.
echo Sandbox Reactions did NOT start. The lines above say why; its log: milestones\m0\logs\launcher.launch.%SERVICE%.log
echo Press Enter to close this window.
set /p "_="
exit /b 1
