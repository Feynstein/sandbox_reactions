#!/usr/bin/env bash
# Double-click launcher (Files -> Run as a Program): starts the game, waits for Enter, stops it (PLAYBOOK §8, contract §3.6).
# Optional first argument: another launch.json service (a check runs `bash start.sh game-xvfb` on the private display).
cd "$(dirname "$0")" || exit 1
service="${1:-game}"
asked=""; [ "$service" = "game" ] && asked="--asked"   # the double-click is the ask (R4)
python3 tools/pb/launch.py start "$service" --task launcher $asked
echo "Sandbox Reactions is running. Press Enter to close it."
read -r _
python3 tools/pb/launch.py stop --task launcher
