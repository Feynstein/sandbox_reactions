#!/bin/bash
# M0-T99b — the same three steps as tasks/M0-T99b.server_run.sh, on linux-pc's RTX 5090: R21's box, the reading Verify 1's Pass
# names (the lead, 2026-11-27, at M0-T99b: «give me the command line to start the 5090 run, ill start it when I leave»). Started by
# the lead in tmux `m0t99b_lpc` from ~/private/sandbox_reactions; read back by the agent that resumes M0-T99b (tasks/M0-T99b.md
# § For the agent who resumes). The task id is M0-T99b-lpc — the 4090's run is M0-T99b-r2 and the two boxes' logs and records
# never share a name (M0-V15-lpc's pattern); its start was recorded with fit.rs's session-2 comment undone, so the claim sees it:
#   1. Verify 1 — `verify.py fit --task M0-T99b-lpc`: three lives to their endings (≈ 1 h on this GPU, read from the 4090's rate);
#   2. Verify 2 — `verify.py --redarm fit --task M0-T99b-lpc`: plant preset-wide (M0-D225's drop at (300, 200)), NO-GO expected —
#      M0-D225's owed arm on its own box; skipped when step 1 is not GO (the arm refuses without a green run);
#   3. the claim — `verify.py --changed --task M0-T99b-lpc` (R10, on its own box: the boxed scopes run here); exit 1 is read, never
#      a stop.
# Progress: build/m0-t99b-lpc/steps.txt (one line per finished step: number, name, exit, time),
# milestones/m0/logs/M0-T99b-lpc.run.log (all output), build/m0-t99b-lpc/run.done when it ends, whatever ended it.
# Restart from a step: START=<step number> bash milestones/m0/tasks/M0-T99b.lpc_run.sh
cd ~/private/sandbox_reactions || exit 1
export SR_TEST_ADAPTER="RTX 5090"
OUT=build/m0-t99b-lpc
LOG=milestones/m0/logs/M0-T99b-lpc.run.log
START=${START:-1}
TASK=M0-T99b-lpc
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
main() {
  stamp "M0-T99b linux-pc run start on $(hostname) (TASK=$TASK, START=$START, SR_TEST_ADAPTER=$SR_TEST_ADAPTER)"
  [ "$(hostname)" = laserax-ai ] || { stamp "refused: this script is linux-pc's (laserax-ai, the RTX 5090)"; return 1; }
  nvidia-smi --query-gpu=name,utilization.gpu,memory.used --format=csv,noheader
  sha256sum assets/calibration.json crates/sr-engine/tests/gpu/fit.rs tests/plants/preset-wide.patch tests/gpu.sh \
    milestones/m0/logs/.baseline/$TASK.json
  step 1 verify_fit python3 tools/pb/verify.py fit --task "$TASK"
  local fit_rc=$?
  if [ "$fit_rc" -eq 0 ]; then
    step 2 redarm_fit python3 tools/pb/verify.py --redarm fit --task "$TASK"
  else
    stamp "step 2 redarm_fit skipped: step 1 exit=$fit_rc (the arm needs a green fit on these bytes)"
    echo "2 redarm_fit skipped(step1_exit=$fit_rc) $(date '+%Y-%m-%dT%H:%M:%S%z')" >> "$OUT/steps.txt"
  fi
  step 3 claim python3 tools/pb/verify.py --changed --task "$TASK"
  stamp "M0-T99b linux-pc run end"
}
main 2>&1 | tee -a "$LOG"
date '+%Y-%m-%dT%H:%M:%S%z' > "$OUT/run.done"
