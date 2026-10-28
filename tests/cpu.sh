#!/usr/bin/env bash
# A CPU scope's runner (R11): bash tests/cpu.sh <crate> <module> — `cargo test` of that crate's lib, the tests of
# `<module>::`. The count line is cargo's own `test result:` (the scope's regex parse). Sources tests/cargo.sh for the
# PATH and the red-arm target. sr-app is bin-only: its unit tests are `--bin sandbox-reactions`, and a check that
# needs the bin's source includes it by #[path] in an integration test — `bash tests/cpu.sh sr-app --test <file>`
# runs that one test binary (G-STR, M0-T13); `bash tests/cpu.sh sr-app --bin <module>` runs the bin's own unit
# tests of `<module>::` (scope `formats`, M0-T75).
source "$(dirname "${BASH_SOURCE[0]}")/cargo.sh"
crate="${1:-}"
module="${2:-}"
[ -n "$crate" ] && [ -n "$module" ] || { echo "=== NO-GO: usage: bash tests/cpu.sh <crate> <module> | <crate> --test <file> | <crate> --bin <module> ==="; exit 1; }
if [ "$module" = "--bin" ]; then
  file="${3:-}"
  [ -n "$file" ] || { echo "=== NO-GO: usage: bash tests/cpu.sh <crate> --bin <module> ==="; exit 1; }
  cd "$_sr_root" && cargo test --release --locked -p "$crate" --bins "${file}::"
elif [ "$module" = "--test" ]; then
  file="${3:-}"
  [ -n "$file" ] || { echo "=== NO-GO: usage: bash tests/cpu.sh <crate> --test <file> ==="; exit 1; }
  cd "$_sr_root" && cargo test --release --locked -p "$crate" --test "$file"
else
  cd "$_sr_root" && cargo test --release --locked -p "$crate" --lib "${module}::"
fi
