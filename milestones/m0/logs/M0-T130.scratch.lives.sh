#!/bin/bash
# M0-T130 Verify 3: the three lives on the RTX 4090, at once, each capped under 10 minutes (SIGINT at 570 s).
# Massive: steps 0–2,000 dumps every 20 (the hold's onset and releases), then --continue to its ending every 250.
# Repro: M0-D152's 20 M0 from s_1362, 1,700 steps (to step 3,062), dumps every 100.
# Life 01: calibrate's m_bh_sb life at 4,572.394, its checkpoints hard-linked as written (calibrate prunes all but the last).
cd ~/workspace/sandbox_reactions || exit 1
B=build/target/release/sandbox-reactions
out=/tmp/t130
rm -rf "$out" && mkdir -p "$out/keep"
T0=$(date +%s)
(
  timeout -s INT 570 $B headless --scene preset:massive --steps 2000 --dump-every 20 --adapter "RTX 4090" --out $out/massive_a > $out/massive_a.out 2>&1
  echo "massive_a exit $? wall $(( $(date +%s) - T0 )) s" >> $out/massive_a.out
  left=$(( 570 - ($(date +%s) - T0) ))
  timeout -s INT $left $B headless --scene preset:massive --load-dump $out/massive_a/dumps/s_2000.bin --continue --until ending --max-steps 60000 --dump-every 250 --adapter "RTX 4090" --out $out/massive_b > $out/massive_b.out 2>&1
  echo "massive_b exit $? wall $(( $(date +%s) - T0 )) s" >> $out/massive_b.out
) &
(
  timeout -s INT 570 $B headless --scene preset:giant --load-dump /tmp/d71f/lives/keep/m_bh_sb-02/s_1362.bin --steps 1700 --dump-every 100 --adapter "RTX 4090" --out $out/repro > $out/repro.out 2>&1
  echo "repro exit $? wall $(( $(date +%s) - T0 )) s" >> $out/repro.out
) &
before=$(ls build/calibrate_runs)
(
  while sleep 0.5; do
    for d in build/calibrate_runs/*/m_bh_sb-01/checkpoint; do
      [ -d "$d" ] || continue
      case "$before" in *"$(basename "$(dirname "$(dirname "$d")")")"*) continue ;; esac
      for f in "$d"/s_*; do [ -e "$f" ] && ln -f "$f" "$out/keep/" 2>/dev/null; done
    done
    [ -e "$out/done" ] && break
  done
) &
watcher=$!
timeout -s INT 570 $B calibrate --only m_bh_sb --bracket 4572.394,4572.394 --adapter "RTX 4090" --out "$out/cal.json" --checkpoint-every 250 --progress-every 250 > "$out/life01.out" 2>&1
echo "life01 exit $? wall $(( $(date +%s) - T0 )) s" >> "$out/life01.out"
touch "$out/done"
wait
ls build/calibrate_runs | tail -1 > "$out/stamp"
echo "all done wall $(( $(date +%s) - T0 )) s"
