#!/usr/bin/env bash
# Double-click launcher (Files -> Run as a Program): starts the game, waits for Enter, stops it (PLAYBOOK §8, contract §3.6).
# Optional first argument: another launch.json service (a check runs `bash start.sh game-xvfb` on the private display).
cd "$(dirname "$0")" || exit 1
service="${1:-game}"
asked=""; [ "$service" = "game" ] && asked="--asked"   # the double-click is the ask (R4)
python3 tools/pb/launch.py start "$service" --task launcher $asked
rc=$?
if [ $rc -ne 0 ]; then   # a start that failed is not a game running: say so, name its log, and ask for Enter only to close this window (M0-D129)
  echo "Sandbox Reactions did NOT start (launch.py start exit $rc). The lines above say why; its log: milestones/m0/logs/launcher.launch.$service.log"
  echo "Press Enter to close this window."
  read -r _
  exit $rc
fi
echo "Sandbox Reactions is running. Press Enter to close it."
read -r _
python3 tools/pb/launch.py stop "$service" --task launcher   # its own service only: a bare stop takes every service on the box (M0-D28)
