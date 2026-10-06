#!/usr/bin/env bash
# Run kernel evals.
# Usage: evals/run.sh [--repeat N] [--jobs J] [case-name ...]   (default: all cases, N=1, J=4)
#
# Each case is evals/cases/<name>/ with:
#   prompt.md   the user prompt (required)
#   fixture/    files copied into a fresh git repo (optional)
#   setup.sh    run in the repo after the first commit (optional)
#   check.sh    deterministic checks, sourced with lib.sh helpers (optional)
#   rubric.md   criteria for a grader model to judge the transcript (optional)
#
# The run repo gets the kernel the way a real repo does: KERNEL.md imported
# from AGENTS.md, and each verb linked into .claude/skills. Only project
# settings load, so user-level skills and memory don't interfere.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EVALS="$ROOT/evals"
REPEAT=1
JOBS=4
while [ $# -gt 0 ]; do
  case "$1" in
    --repeat) REPEAT="$2"; shift 2 ;;
    --jobs) JOBS="$2"; shift 2 ;;
    --one) ONE=1; shift ;;
    *) break ;;
  esac
done

digest() {
  jq -r '
    if .type=="assistant" then .message.content[]?
      | if .type=="text" then "ASSISTANT: " + .text
        elif .type=="tool_use" then "TOOL " + .name + ": " + (.input|tostring|.[0:500])
        else empty end
    elif .type=="user" then .message.content[]?
      | select(type=="object" and .type=="tool_result")
      | "RESULT: " + (.content|tostring|.[0:500])
    else empty end' "$1"
}

run_case() {
  local name="$1" rep="$2" src="$EVALS/cases/$1" dir="$OUT/$1/r$2"
  mkdir -p "$dir/repo"
  local repo="$dir/repo"

  [ -d "$src/fixture" ] && cp -R "$src/fixture/." "$repo/"
  cp "$ROOT/kernel/KERNEL.md" "$repo/KERNEL.md"
  echo "@KERNEL.md" > "$repo/AGENTS.md"
  mkdir -p "$repo/.claude/skills"
  for v in "$ROOT"/kernel/*/; do ln -s "$v" "$repo/.claude/skills/$(basename "$v")"; done
  (cd "$repo" && git init -q && git add -A && git commit -qm "fixture")
  [ -f "$src/setup.sh" ] && (cd "$repo" && bash "$src/setup.sh")

  (cd "$repo" && claude -p "$(cat "$src/prompt.md")" \
      --setting-sources project \
      --output-format stream-json --verbose \
      --permission-mode acceptEdits \
      --allowedTools "Bash(git:*) Bash(python3:*) Bash(ls:*) Bash(cat:*) Bash(mkdir:*) Bash(grep:*) Bash(find:*)" \
      --disallowedTools "AskUserQuestion" \
      > "$dir/transcript.jsonl" 2> "$dir/stderr.log")
  digest "$dir/transcript.jsonl" > "$dir/digest.txt"

  local result="PASS" note=""
  if [ -f "$src/check.sh" ]; then
    note="$(cd "$repo" && TRANSCRIPT="$dir/transcript.jsonl" bash -c "source '$EVALS/lib.sh'; source '$src/check.sh'" 2>&1)" \
      || result="FAIL"
  fi

  if [ "$result" = "PASS" ] && [ -f "$src/rubric.md" ]; then
    local work=""
    [ -d "$repo/work" ] && work="$(cd "$repo" && find work -type f -exec sh -c 'echo "=== $1"; cat "$1"' _ {} \;)"
    local grader_dir; grader_dir="$(mktemp -d)"
    (cd "$grader_dir" && claude -p --setting-sources project "You are grading one run of an agent skill. Judge only against the rubric. For each criterion give PASS or FAIL with one line of evidence quoted from the transcript or files. End with exactly one line: VERDICT: PASS or VERDICT: FAIL (FAIL if any criterion fails).

<rubric>
$(cat "$src/rubric.md")
</rubric>

<transcript>
$(cat "$dir/digest.txt")
</transcript>

<work-files>
$work
</work-files>") > "$dir/grade.md" 2>&1
    grep -q "^VERDICT: PASS" "$dir/grade.md" || { result="FAIL"; note="rubric: see grade.md"; }
  fi

  printf "%-28s r%-3s %s %s\n" "$name" "$rep" "$result" "$note" | tee -a "$OUT/results.txt"
}

# Worker mode: xargs re-invokes this script once per (case, rep).
if [ "${ONE:-}" = 1 ]; then run_case "$1" "$2"; exit 0; fi

OUT="$EVALS/.runs/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$OUT"
export OUT

cases=("$@")
[ ${#cases[@]} -eq 0 ] && cases=($(ls "$EVALS/cases"))

for c in "${cases[@]}"; do
  for r in $(seq 1 "$REPEAT"); do echo "$c $r"; done
done | xargs -n 2 -P "$JOBS" "$0" --one

echo
echo "Summary (passes / runs):"
for c in "${cases[@]}"; do
  printf "  %-28s %s/%s\n" "$c" "$(grep -c "^$c .* PASS" "$OUT/results.txt")" "$REPEAT"
done
echo "Runs: $OUT"
! grep -q " FAIL" "$OUT/results.txt"
