#!/bin/bash
# M0-V15 — the whole recalibration and every run this gate reads, on laserax-ai-server's RTX 4090 (the lead, 2026-11-25, at M0-V15:
# «Whole, from item 1 (Recommended)», «give me something I can paste on the server thats running in tmux», then «make it so
# everything runs without having me to paste anything»). Started by M0-V15's session in tmux `m0v15`, from the repo root of
# ~/workspace/sandbox_reactions; read back by the agent that resumes M0-V15 (tasks/M0-V15.md § For the agent who resumes). The
# pattern is M0-T133's server run (tasks/M0-T133.server_run.sh), from item 1 this time — M0-D212 changed P4's update, so every GPU
# life is re-measured:
#   1. build (the tree rsync'd from linux-pc, M0-D210–D219's work in it);
#   2–6. the calibration into a fresh build/m0-v15/calibration.json: items 1–2 (CPU), m_ign_sb, m_up_sb, m_bh_sb, then the presets
#      and the items after them; an item that fails stops the run there (exit 6 is a D, never a re-tune);
#   7. the file copied into assets/ (the 2026-11-23 file kept beside it) and embedded by a rebuild;
#   8. the new calibration accepted (a headless step, exit 0 expected);
#   9. every scope on the recalibrated tree, the ⏱ lives included — `verify.py --all --task M0-V15`; boxed scopes (desktop, web,
#      capture, measure_ui, toolchain) read owed here; exit 1 is read, never a stop;
#  10. the red-arm sweep, `verify.py --redarm --moved --task M0-V15`.
# Progress: build/m0-v15/steps.txt (one line per finished step: number, name, exit, time), milestones/m0/logs/M0-V15.server.log (all
# output), build/m0-v15/run.done when it ends, whatever ended it. Restart from a step: START=<step number> bash
# milestones/m0/tasks/M0-V15.server_run.sh (a stopped calibration item adds --resume build/calibrate_runs/<its stamp> to its line —
# a stamp of this run only).
cd ~/workspace/sandbox_reactions || exit 1
export SR_TEST_ADAPTER="RTX 4090"
OUT=build/m0-v15
LOG=milestones/m0/logs/M0-V15.server.log
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
install_cal() {
  [ -f "$OUT/calibration.2026-11-23.json" ] || cp assets/calibration.json "$OUT/calibration.2026-11-23.json"
  cp "$OUT/calibration.json" assets/calibration.json && cargo build --release --locked -p sr-app
}
summary() {
  python3 -c 'import json; c=json.load(open("assets/calibration.json")); print({k: c.get(k) for k in ("physics_hash","measured","m_ign_sb","m_ch_sb","m_up_sb","m_bh_sb","m_tov_sb","preset_mass_sb","life_sun_sb","top_rung","max_gas_speed")})'
}
main() {
  stamp "M0-V15 server run start on $(hostname) (START=$START, SR_TEST_ADAPTER=$SR_TEST_ADAPTER)"
  sha256sum assets/elements.json assets/reactions.json assets/physics.json assets/calibration.json crates/sr-engine/shaders/hydro.wgsl crates/sr-engine/src/step/hydro.rs
  source tests/cargo.sh || { stamp "tests/cargo.sh FAILED"; return 1; }
  step 1 build cargo build --release --locked -p sr-app || return 1
  step 2 cal_items_1_2 cal --only m_ch_sb,m_tov_sb &&
  step 3 cal_m_ign cal --only m_ign_sb --progress-every 1000 &&
  step 4 cal_m_up cal --only m_up_sb --progress-every 5000 --max-steps 600000 &&
  step 5 cal_m_bh cal --only m_bh_sb --progress-every 2000 --max-steps 400000 &&
  step 6 cal_presets cal --only preset_mass_sb --progress-every 2000 --max-steps 600000 ||
    { stamp "calibration STOPPED — nothing after it runs (restart: START=<the failed step>, a stopped item adds --resume build/calibrate_runs/<its stamp>)"; return 1; }
  step 7 install_calibration install_cal || return 1
  summary
  step 8 accepted build/target/release/sandbox-reactions headless --scene preset:sun --steps 1 --out "$OUT/accepted" --adapter "RTX 4090" || return 1
  step 9 verify_all python3 tools/pb/verify.py --all --task M0-V15
  step 10 redarm_moved python3 tools/pb/verify.py --redarm --moved --task M0-V15
  stamp "M0-V15 server run end"
}
main 2>&1 | tee -a "$LOG"
date '+%Y-%m-%dT%H:%M:%S%z' > "$OUT/run.done"
