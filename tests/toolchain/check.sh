#!/usr/bin/env bash
# The `toolchain` scope's case runner (M0-TH): check.sh desktop|wasm|trunk
#   desktop  rustc 1.99 builds the one-file program natively and it prints its answer
#   wasm     the same crate builds for wasm32-unknown-unknown into a real .wasm module
#   trunk    trunk 0.21.14 builds the web page around it (wasm-bindgen glue, index.html)
# Builds go to build/ (verify.py's scratch copies leave that folder out). Ends in the Rule 4 verdict.
set -u
case_="${1:-}"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TARGET_DIR="$root/build/toolchain-target"
compiler=cargo
web_target=wasm32-unknown-unknown
bundler=trunk

die() { echo "toolchain $case_: $1"; echo "=== NO-GO: $1 ==="; exit 1; }
ok() { echo "toolchain $case_: $1"; echo "=== GO ==="; exit 0; }

command -v "$compiler" >/dev/null 2>&1 || die "$compiler not found on PATH (~/.cargo/bin is put first here; is Rust installed?)"
cd "$here" || die "cannot enter $here"
case "$case_" in
  desktop)
    rustc --version | grep -q '^rustc 1\.99\.' || die "rustc is not 1.99 (contract §1.2): $(rustc --version)"
    "$compiler" build --locked --quiet >/dev/null 2>&1 || die "$compiler build failed"
    out="$("$CARGO_TARGET_DIR/debug/sr-toolchain" 2>&1)" || die "the built program did not run"
    [ "$out" = "sr-toolchain 42" ] || die "the built program printed '$out'"
    ok "$(rustc --version) built and ran the one-file program"
    ;;
  wasm)
    "$compiler" build --locked --quiet --target "$web_target" >/dev/null 2>&1 || die "$compiler build --target $web_target failed"
    wasm="$CARGO_TARGET_DIR/$web_target/debug/sr-toolchain.wasm"
    [ -f "$wasm" ] || die "no $wasm"
    [ "$(head -c 4 "$wasm" | od -An -c | tr -d ' ')" = '\0asm' ] || die "$wasm is not a wasm module"
    ok "$web_target module built ($(stat -c %s "$wasm") bytes)"
    ;;
  trunk)
    command -v "$bundler" >/dev/null 2>&1 || die "$bundler not found on PATH"
    "$bundler" --version | grep -q '^trunk 0\.21\.14' || die "trunk is not 0.21.14 (contract §1.2): $("$bundler" --version)"
    dist="$root/build/toolchain-dist"
    mkdir -p "$root/build" || die "cannot create $root/build"
    "$bundler" build --dist "$dist" >"$root/build/toolchain-trunk.log" 2>&1 || die "$bundler build failed (build/toolchain-trunk.log)"
    ls "$dist"/*.wasm >/dev/null 2>&1 || die "no .wasm in $dist"
    ls "$dist"/*.js >/dev/null 2>&1 || die "no wasm-bindgen .js glue in $dist"
    [ -f "$dist/index.html" ] || die "no index.html in $dist"
    ok "$("$bundler" --version) built the page"
    ;;
  *) die "unknown case '$case_' (desktop, wasm or trunk)" ;;
esac
