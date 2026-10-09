@echo off
rem Double-click launcher for Windows: starts the game, waits for Enter, stops it (PLAYBOOK section 8, contract section 3.6 [M0-TJ3]).
rem Optional first argument: another launch.json service (a check runs "start.bat game-offscreen").
cd /d "%~dp0"
set "SERVICE=%~1"
if "%SERVICE%"=="" set "SERVICE=game"
set "ASKED="
if "%SERVICE%"=="game" set "ASKED=--asked"
py -3.12 tools\pb\launch.py start %SERVICE% --task launcher %ASKED%
echo Sandbox Reactions is running. Press Enter to close it.
set /p "_="
py -3.12 tools\pb\launch.py stop --task launcher
