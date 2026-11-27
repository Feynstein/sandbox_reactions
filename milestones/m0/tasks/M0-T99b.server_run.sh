#!/bin/bash
# M0-T99b — G-FIT's Verify 1, Verify 2 and the claim, on laserax-ai-server's RTX 4090 (the lead, 2026-11-27, at M0-T99b: «Give me a
# tmux command line, if thats whats needed, to start everything on the server in a tmux session so that it can run over the
# weekend»). The pattern is M0-V15's server run (tasks/M0-V15.server_run.sh), with no calibration: 59b7bb46… is installed on both
# boxes (M0-V15). Started from linux-pc by the one line in tasks/M0-T99b.md § The server run (the tree rsync'd, then tmux `m0t99b`);
# read back by the agent that resumes M0-T99b (tasks/M0-T99b.md § For the agent who resumes):
# The task id is M0-T99b-r2 (session 1's logs, M0-T99b.*, are committed — «a re-run takes a fresh `--task` id», m0_rules.md
# Hazards 2026-10-08); its start was recorded on linux-pc before session 2's fit.rs edit.
#   1. Verify 1 — `verify.py fit --task M0-T99b-r2`: three lives to their endings (≈ 4 h on this GPU: M0-V15's fit read 14,525 s);
#   2. Verify 2 — `verify.py --redarm fit --task M0-T99b-r2`: plant preset-wide (M0-D225's drop at (300, 200)), NO-GO expected — the
#      same arm M0-D225 owes, on the same plant and bytes; skipped when step 1 is not GO (the arm refuses without a green run);
#   3. the claim — `verify.py --changed --task M0-T99b-r2`, read against the start recorded on linux-pc
#      (milestones/m0/logs/.baseline/M0-T99b-r2.json, rsync'd with the tree); boxed scopes (desktop, web, capture, measure_ui) read
#      owed here; exit 1 is read, never a stop.
# Progress: build/m0-t99b/steps.txt (one line per finished step: number, name, exit, time), milestones/m0/logs/M0-T99b.server.log
# (all output), build/m0-t99b/run.done when it ends, whatever ended it. Restart from a step: START=<step number> bash
# milestones/m0/tasks/M0-T99b.server_run.sh
cd ~/workspace/sandbox_reactions || exit 1
export SR_TEST_ADAPTER="RTX 4090"
OUT=build/m0-t99b
LOG=milestones/m0/logs/M0-T99b.server.log
START=${START:-1}
TASK=M0-T99b-r2
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
  stamp "M0-T99b server run start on $(hostname) (TASK=$TASK, START=$START, SR_TEST_ADAPTER=$SR_TEST_ADAPTER)"
  [ "$(hostname)" = laserax-ai-server ] || { stamp "refused: this script is laserax-ai-server's (the RTX 4090)"; return 1; }
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
  stamp "M0-T99b server run end"
}
main 2>&1 | tee -a "$LOG"
date '+%Y-%m-%dT%H:%M:%S%z' > "$OUT/run.done"
