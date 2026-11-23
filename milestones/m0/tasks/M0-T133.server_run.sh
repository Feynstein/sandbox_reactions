#!/bin/bash
# M0-T133 — the recalibration's restart and every run owed to M0-V15, on laserax-ai-server's RTX 4090 (the lead, 2026-11-23, at
# M0-D209: «start the recalibration and all the other tasks in a tmux terminal on laserax-ai-server. Verify that the last one is
# killed before you start your own»). Started by M0-D209's session in tmux `m0t133`, from the repo root of ~/workspace/sandbox_reactions;
# read back by M0-T133 (the plan's block). The pattern is M0-D203's server run (tasks/M0-D203.server_run.sh), restarted per M0-D208's
# Verify 3: physics_hash holds while the sinks' shader moved (the Y_e gate 0.35 → 0.40), so items 1–4 stand and every m_bh life is
# re-decided:
#   1. build (the tree rsync'd from linux-pc, M0-D208's shader in it);
#   2. m_bh_sb re-measured into build/m0-t133/calibration.json, whose base is M0-D203's server file (items 1–5 on 800 × 800, the same
#      RTX 4090) — `--only` keeps the rest (§3.3 [M0-D205]: same physics_hash, same world);
#   3. the presets (preset_mass_sb and the items after it); an item that fails stops the run there (exit 6 is a D, never a re-tune);
#   4. the file copied into assets/ (the 2026-11-13 file kept beside it) and embedded by a rebuild;
#   5. the new calibration accepted (a headless step, exit 0 expected);
#   6. every scope on the recalibrated tree, the ⏱ lives included — the claims, lives and scopes owed to M0-V15 (fit's three lives =
#      M0-T99b's Verify 1) — `verify.py --all --task M0-V15`; boxed scopes (desktop, web, capture, measure_ui, toolchain) read owed here;
#   7. the red-arm sweep, `verify.py --redarm --moved --task M0-V15` (calibration, fit, held_core, conservation, touch, endings, squeeze …).
# Progress: build/m0-t133/steps.txt (one line per finished step: name, exit), milestones/m0/logs/M0-T133.server.log (all output),
# build/m0-t133/run.done when it ends. Restart from a step: START=<step number> bash milestones/m0/tasks/M0-T133.server_run.sh
# (a stopped calibration item adds --resume build/calibrate_runs/<its stamp> to its line — a stamp of this run only).
cd ~/workspace/sandbox_reactions || exit 1
export SR_TEST_ADAPTER="RTX 4090"
OUT=build/m0-t133
BASE=build/m0-d203/calibration.json
LOG=milestones/m0/logs/M0-T133.server.log
START=${START:-1}
mkdir -p "$OUT"
stamp() { echo "=== $1 · $(date '+%Y-%m-%dT%H:%M:%S%z')"; }
step() {
  local n=$1 name=$2; shift 2
  if [ "$n" -lt "$START" ]; then stamp "step $n $name skipped (START=$START)"; return 0; fi
  stamp "step $n $name start"
  "$@"; local rc=$?
  stamp "step $n $name end exit=$rc"
  echo "$n $name exit=$rc $(date '+%Y-%m-%dT%H:%M:%S%z')" >> "$OUT/steps.txt"
  return $rc
}
cal() { build/target/release/sandbox-reactions calibrate --adapter "RTX 4090" --out "$OUT/calibration.json" --checkpoint-every 3000 "$@"; }
base_cal() { [ -f "$OUT/calibration.json" ] || cp "$BASE" "$OUT/calibration.json"; }
install_cal() {
  [ -f "$OUT/calibration.2026-11-13.json" ] || cp assets/calibration.json "$OUT/calibration.2026-11-13.json"
  cp "$OUT/calibration.json" assets/calibration.json && cargo build --release --locked -p sr-app
}
summary() {
  python3 -c 'import json; c=json.load(open("assets/calibration.json")); print({k: c.get(k) for k in ("physics_hash","measured","m_ign_sb","m_ch_sb","m_up_sb","m_bh_sb","m_tov_sb","preset_mass_sb","life_sun_sb","top_rung","max_gas_speed")})'
}
main() {
  stamp "M0-T133 server run start on $(hostname) (START=$START, SR_TEST_ADAPTER=$SR_TEST_ADAPTER)"
  sha256sum assets/elements.json assets/reactions.json assets/physics.json assets/calibration.json crates/sr-engine/shaders/sinks.wgsl "$BASE"
  source tests/cargo.sh || { stamp "tests/cargo.sh FAILED"; return 1; }
  step 1 build cargo build --release --locked -p sr-app || return 1
  base_cal || { stamp "no base $BASE"; return 1; }
  step 2 cal_m_bh cal --only m_bh_sb --progress-every 2000 --max-steps 400000 &&
  step 3 cal_presets cal --only preset_mass_sb --progress-every 2000 --max-steps 600000 ||
    { stamp "calibration STOPPED — nothing after it runs (restart: START=<the failed step>, a stopped item adds --resume build/calibrate_runs/<its stamp>)"; return 1; }
  step 4 install_calibration install_cal || return 1
  summary
  step 5 accepted build/target/release/sandbox-reactions headless --scene preset:sun --steps 1 --out "$OUT/accepted" --adapter "RTX 4090"
  step 6 verify_all python3 tools/pb/verify.py --all --task M0-V15
  step 7 redarm_moved python3 tools/pb/verify.py --redarm --moved --task M0-V15
  stamp "M0-T133 server run end"
}
main 2>&1 | tee -a "$LOG"
touch "$OUT/run.done"
