#!/bin/bash
# M0-D203 — the lead's weekend run on laserax-ai-server's RTX 4090 (the lead, 2026-11-20: «start that run in a tmux terminal on
# laserax-ai-server … make it so all the owed tests are run one after the others, the ones that were owed to the next v gate …
# Use a fire and forget tmux»). Started by M0-D203 in tmux `m0d203`, from the repo root of ~/workspace/sandbox_reactions.
#   1. the recalibration on the 800 × 800 default, into a fresh --out (never the 600 × 400 file as base, M0-D205), then copied into
#      assets/ (the old file kept beside it) and embedded by a rebuild; a calibration item that fails stops the run there;
#   2. the new calibration accepted (Verify 3's command, now expecting exit 0);
#   3. every scope on the recalibrated tree, the ⏱ lives included — the claims, lives and scopes owed to M0-V15
#      (`verify.py --all --task M0-V15`);
#   4. the red-arm sweep (`verify.py --redarm --moved --task M0-V15`) — the owed red-arms (calibration, held_core, stages,
#      conservation, touch, endings, squeeze, fit …), each on its green run.
# Progress: build/m0-d203/steps.txt (one line per finished step: name, exit), milestones/m0/logs/M0-D203.server.log (all output),
# build/m0-d203/run.done when it ends. Restart from a step: START=<step number> bash milestones/m0/tasks/M0-D203.server_run.sh
cd ~/workspace/sandbox_reactions || exit 1
export SR_TEST_ADAPTER="RTX 4090"
OUT=build/m0-d203
LOG=milestones/m0/logs/M0-D203.server.log
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
  [ -f "$OUT/calibration.2026-11-13.json" ] || cp assets/calibration.json "$OUT/calibration.2026-11-13.json"
  cp "$OUT/calibration.json" assets/calibration.json && cargo build --release --locked -p sr-app
}
summary() {
  python3 -c 'import json; c=json.load(open("assets/calibration.json")); print({k: c.get(k) for k in ("physics_hash","measured","m_ign_sb","m_ch_sb","m_up_sb","m_bh_sb","m_tov_sb","preset_mass_sb","life_sun_sb","top_rung","max_gas_speed")})'
}
main() {
  stamp "M0-D203 server run start on $(hostname) (START=$START, SR_TEST_ADAPTER=$SR_TEST_ADAPTER)"
  sha256sum assets/elements.json assets/reactions.json assets/physics.json assets/calibration.json
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
  step 8 accepted build/target/release/sandbox-reactions headless --scene preset:sun --steps 1 --out "$OUT/accepted" --adapter "RTX 4090"
  step 9 verify_all python3 tools/pb/verify.py --all --task M0-V15
  step 10 redarm_moved python3 tools/pb/verify.py --redarm --moved --task M0-V15
  stamp "M0-D203 server run end"
}
main 2>&1 | tee -a "$LOG"
touch "$OUT/run.done"
