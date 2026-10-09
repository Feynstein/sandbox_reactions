#!/usr/bin/env bash
# A GPU scope's runner (R11): bash tests/gpu.sh <module> [physics] — `cargo test` of sr-engine's one GPU binary, the
# tests of `<module>::`, one at a time, output shown so the log names the adapters (R13). The count line is cargo's own
# `test result:` (the scope's regex parse). Sources tests/cargo.sh for the PATH and the red-arm target.
# `physics`: the module's numbers come from the box's physics adapter (R12; R15) — SR_TEST_ADAPTER is set to the RTX
# 5090, on Windows the RTX 4080 Laptop, unless the caller set it; wgpu's own pick on linux-pc is the Quadro (M0-T2).
source "$(dirname "${BASH_SOURCE[0]}")/cargo.sh"
module="${1:-}"
[ -n "$module" ] || { echo "=== NO-GO: usage: bash tests/gpu.sh <module> [physics] ==="; exit 1; }
if [ "${2:-}" = "physics" ] && [ -z "${SR_TEST_ADAPTER:-}" ]; then
  case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) export SR_TEST_ADAPTER="RTX 4080" ;;
    *) export SR_TEST_ADAPTER="RTX 5090" ;;
  esac
fi
cd "$_sr_root" && cargo test --release --locked -p sr-engine --test gpu "${module}::" -- --nocapture --test-threads=1
