#!/usr/bin/env bash
# tools/pb/scratch_copy.sh — the sanctioned scratch copy (PLAYBOOK annex §A.7).
#
#   scratch_copy.sh <src> <dest> [--also-exclude <name>]…   a copy of the tree to plant a bug in, or run a mutant
#   scratch_copy.sh --selftest                              red-armed selftest in a temp folder
#
# rsync -a <src>/ <dest>/ without the folders named below (matched by name at any depth) and the extras;
# images/ folders are kept empty (plan.py lint wants one beside every plan); then .venv and node_modules
# are symlinked from <src> when they exist there, so the copy still runs.
# <dest> must be absent or empty and must not sit inside <src>; nothing is ever deleted, git never runs.
# Output: one counts line (dest, files, size, links), then === GO === or === NO-GO: <reason> === (exit 0 / 1);
# a copy of zero files is NO-GO (annex §A.0: the verdict is derived from the counts).
set -uo pipefail

EXCLUDES=(.git .venv venv node_modules models data var output __pycache__ .pytest_cache dist build)
EMPTIED=(images)                            # the folder is copied, its contents are not
LINKS=(.venv node_modules)

nogo() { echo "files=0 · dest=${2:--}"; echo "=== NO-GO: $1 ==="; return 1; }

copy() {                                    # copy <src> <dest> [extra names…]
  local src dest name files ex=() links=()
  src="$(cd "$1" 2>/dev/null && pwd -P)" || { nogo "no source folder: $1" "$2"; return 1; }
  dest="$(realpath -m -- "$2")" || { nogo "cannot resolve $2" "$2"; return 1; }
  [[ "$dest/" == "$src/"* ]] && { nogo "dest sits inside the source: $dest" "$dest"; return 1; }
  [[ -e "$dest" && -n "$(ls -A "$dest" 2>&1)" ]] && { nogo "dest is not empty: $dest — use a fresh folder" "$dest"; return 1; }
  command -v rsync >/dev/null || { nogo "rsync not found" "$dest"; return 1; }
  mkdir -p "$dest" || { nogo "cannot create $dest" "$dest"; return 1; }
  shift 2
  for name in "${EXCLUDES[@]}" "$@"; do ex+=("--exclude=$name"); done
  for name in "${EMPTIED[@]}"; do ex+=("--exclude=$name/*"); done
  rsync -a "${ex[@]}" "$src/" "$dest/" || { nogo "rsync failed (exit $?)" "$dest"; return 1; }
  for name in "${LINKS[@]}"; do
    [[ -e "$src/$name" ]] && ln -s "$src/$name" "$dest/$name" && links+=("$name")
  done
  files=$(find "$dest" -type f | wc -l)
  echo "dest=$dest · files=$files · size=$(du -sh "$dest" | cut -f1) · linked=${links[*]:-none} · excluded=${#ex[@]}"
  [[ $files -gt 0 ]] || { echo "=== NO-GO: zero files copied — every file of the source sits in an excluded folder ==="; return 1; }
  echo "=== GO ==="
}

selftest() {
  local tmp out rc name checks=0 failed=0 fails=() keep=()
  # the fixture follows annex §A.7's list, written out here — never EXCLUDES, or a name dropped there drops here too
  local spec=(.git .venv venv node_modules models data var output images __pycache__ .pytest_cache dist build)
  ok() { checks=$((checks+1)); [[ $2 -eq 0 ]] || { failed=$((failed+1)); fails+=("$1"); }; }
  leaked() { find "$1" -name planted -type f | wc -l; }      # excluded files that reached a copy (links not followed)
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/scratch_copy_selftest.XXXXXX")" || { echo "checks=0"; echo "=== NO-GO: no temp folder ==="; return 1; }
  trap "rm -rf -- \"$tmp\"" EXIT                              # only what this selftest created, path fixed now
  mkdir -p "$tmp/src/keep" && echo kept > "$tmp/src/keep/kept.txt" && echo kept > "$tmp/src/top.txt"
  for name in "${spec[@]}"; do
    mkdir -p "$tmp/src/$name" "$tmp/src/keep/$name" && echo x > "$tmp/src/$name/planted" && echo x > "$tmp/src/keep/$name/planted"
  done
  out="$(copy "$tmp/src" "$tmp/d1")"; rc=$?
  [[ $rc -eq 0 && "$out" == *"=== GO ==="* ]]; ok "the clean copy: GO, exit 0" $?
  [[ -f "$tmp/d1/keep/kept.txt" ]]; ok "a kept file is copied" $?
  [[ "$(leaked "$tmp/d1")" -eq 0 ]]; ok "no file of §A.7's ${#spec[@]} excluded folders is copied, top level or nested" $?
  [[ -d "$tmp/d1/images" && -d "$tmp/d1/keep/images" && -z "$(find "$tmp/d1/images" "$tmp/d1/keep/images" -mindepth 1)" ]]
  ok "images/ folders are kept, empty (plan.py lint wants one beside every plan)" $?
  [[ -L "$tmp/d1/.venv" && "$(readlink "$tmp/d1/.venv")" == "$(cd "$tmp/src" && pwd -P)/.venv" && -L "$tmp/d1/node_modules" ]]
  ok ".venv and node_modules are symlinks to the source's" $?
  out="$(copy "$tmp/src" "$tmp/d2" keep)"; rc=$?
  [[ $rc -eq 0 && ! -e "$tmp/d2/keep" && -f "$tmp/d2/top.txt" ]]; ok "--also-exclude drops one more folder" $?
  out="$(copy "$tmp/src" "$tmp/d1")"; rc=$?
  [[ $rc -eq 1 && "$out" == *"=== NO-GO: dest is not empty"* ]]; ok "a dest that is not empty → NO-GO" $?
  out="$(copy "$tmp/src" "$tmp/src/inner")"; rc=$?
  [[ $rc -eq 1 && "$out" == *"inside the source"* && ! -e "$tmp/src/inner" ]]; ok "a dest inside the source → NO-GO, nothing created" $?
  out="$(copy "$tmp/nowhere" "$tmp/d4")"; rc=$?
  [[ $rc -eq 1 && ! -e "$tmp/d4" ]]; ok "a missing source → NO-GO, nothing created" $?
  mkdir -p "$tmp/bare/models" && echo x > "$tmp/bare/models/planted"
  out="$(copy "$tmp/bare" "$tmp/d5")"; rc=$?
  [[ $rc -eq 1 && "$out" == *"files=0 "* && "${out##*$'\n'}" == "=== NO-GO: zero files copied"* ]]
  ok "PLANT a source whose one file sits in an excluded folder → files=0 → NO-GO, the last line" $?
  out="$(rsync() { return 23; }; copy "$tmp/src" "$tmp/d6")"; rc=$?
  [[ $rc -eq 1 && "${out##*$'\n'}" == "=== NO-GO: rsync failed (exit 23) ===" ]]; ok "PLANT rsync fails → NO-GO naming its exit code" $?
  for name in "${EXCLUDES[@]}"; do [[ $name == models ]] || keep+=("$name"); done
  out="$(EXCLUDES=("${keep[@]}"); copy "$tmp/src" "$tmp/d3")"; rc=$?
  [[ $rc -eq 0 && "$(leaked "$tmp/d3")" -eq 2 ]]; ok "PLANT \`models\` removed from the list → its two files appear → the leak check turns red" $?
  echo "checks=$checks · passed=$((checks-failed)) · failed=$failed"
  for name in "${fails[@]}"; do echo "FAIL $name"; done
  if [[ $checks -gt 0 && $failed -eq 0 ]]; then echo "=== GO ==="; else echo "=== NO-GO: $failed check(s) failed ==="; return 1; fi
}

case "${1:-}" in
  --selftest) selftest; exit $? ;;
  -h|--help) sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
esac
[[ $# -ge 2 ]] || { nogo "usage: scratch_copy.sh <src> <dest> [--also-exclude <name>]…"; exit 1; }
src="$1"; dest="$2"; shift 2; extra=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --also-exclude) [[ $# -ge 2 ]] || { nogo "--also-exclude needs a name" "$dest"; exit 1; }; extra+=("$2"); shift 2 ;;
    *) nogo "unknown argument: $1" "$dest"; exit 1 ;;
  esac
done
copy "$src" "$dest" "${extra[@]}"
