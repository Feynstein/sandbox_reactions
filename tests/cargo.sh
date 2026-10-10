#!/usr/bin/env bash
# Every cargo scope sources this file (m0_contrat.md §6.5; docs/agent/testing.md § Cargo scopes). Run as
#   bash tests/cargo.sh build native|wasm
# it is the `build` scope's case runner and ends in `=== GO ===` or `=== NO-GO: <reason> ===`.
#   native  cargo build --workspace --release --locked, then cargo clippy ... -- -D warnings
#   wasm    cargo check --workspace --target wasm32-unknown-unknown --locked
# Sourced, it only sets the environment: ~/.cargo/bin first on the PATH (off it on linux-pc, Hazards), and in a
# red-arm scratch copy (no `.git`) a CARGO_TARGET_DIR shared outside the tree, so a plant recompiles only the
# workspace's crates, and a `cargo` wrapper that keeps that sharing honest (M0-D9, below). That shared dir is for test builds only, never for a binary a scope launches (two arms
# would race on one uplifted binary). In the real tree the target is build/target (.cargo/config.toml).
_sr_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
if [ -e "$_sr_root/.git" ]; then
  unset CARGO_TARGET_DIR
  sr_in_shared_target() { "$@"; }
else
  export CARGO_TARGET_DIR="$HOME/.cache/sandbox-reactions/redarm-target"
  # M0-D9: cargo's freshness is mtime-based and the copy keeps each file's mtime, so a build another arm left in the
  # shared target looks fresh here (a clean arm ran the plant's binary). Every cargo call in a copy therefore takes
  # the shared target's lock, touches the workspace's inputs — later than any build already there — and builds
  # inside it; only the workspace's crates recompile, the dependencies stay cached. A tool that runs the cargo binary
  # itself (trunk) bypasses the `cargo` function: it runs as `sr_in_shared_target <tool> ...` (M0-T15 — a clean web
  # arm served the never-ready plant's wasm).
  _sr_lock="$CARGO_TARGET_DIR.lock"
  cargo() { sr_in_shared_target command cargo "$@"; }
  sr_in_shared_target() {
    mkdir -p "${CARGO_TARGET_DIR%/*}"
    (
      if command -v flock >/dev/null 2>&1; then
        exec 9>"$_sr_lock" && flock 9
      else  # no flock (Git Bash): a mkdir lock, broken when its holder is gone
        until mkdir "$_sr_lock.d" 2>/dev/null; do
          local holder; holder="$(cat "$_sr_lock.d/pid" 2>/dev/null)"
          [ -n "$holder" ] && ! kill -0 "$holder" 2>/dev/null && rm -rf "$_sr_lock.d"
          sleep 0.2
        done
        echo "$BASHPID" >"$_sr_lock.d/pid"
        trap 'rm -rf "$_sr_lock.d"' EXIT
      fi
      local p; for p in crates assets scenes Cargo.toml Cargo.lock .cargo; do
        [ -e "$_sr_root/$p" ] && find "$_sr_root/$p" -type f -exec touch {} +
      done
      "$@"
    )
  }
fi

# sr_cargo_run <label> <cargo args...>: run cargo in the workspace root; on failure print its last lines.
sr_cargo_run() {
  local label="$1"; shift
  local out; out="$(mktemp)"
  if (cd "$_sr_root" && cargo "$@") >"$out" 2>&1; then
    rm -f "$out"; return 0
  fi
  echo "--- cargo $* ($label) ---"; tail -n 25 "$out"; rm -f "$out"; return 1
}

sr_build_case() {
  local case_="$1" start=$SECONDS
  local tdir="${CARGO_TARGET_DIR:-$_sr_root/build/target}"
  command -v cargo >/dev/null 2>&1 || { echo "=== NO-GO: cargo not found (is Rust installed? ~/.cargo/bin is put first here) ==="; return 1; }
  case "$case_" in
    native)
      sr_cargo_run build build --workspace --release --locked \
        || { echo "=== NO-GO: cargo build --workspace --release --locked failed ==="; return 1; }
      sr_cargo_run clippy clippy --workspace --release --locked -- -D warnings \
        || { echo "=== NO-GO: cargo clippy -D warnings failed ==="; return 1; }
      ;;
    wasm)
      sr_cargo_run check check --workspace --target wasm32-unknown-unknown --locked \
        || { echo "=== NO-GO: cargo check --target wasm32-unknown-unknown failed ==="; return 1; }
      ;;
    *) echo "=== NO-GO: unknown case '$case_' (native or wasm) ==="; return 1 ;;
  esac
  echo "build $case_: $(cargo --version | cut -d' ' -f2) in $((SECONDS - start)) s, target $tdir ($(du -sh "$tdir" 2>/dev/null | cut -f1))"
  echo "=== GO ==="
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  case "${1:-}" in
    build) sr_build_case "${2:-}" ;;
    *) echo "=== NO-GO: usage: bash tests/cargo.sh build native|wasm ==="; exit 1 ;;
  esac
fi
