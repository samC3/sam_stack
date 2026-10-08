#!/usr/bin/env bash
# Run evals.
# Usage: evals/run.sh [--repeat N] [--jobs J] [--root DIR] [--cases] [name ...]
#        evals/run.sh --graders [task ...]
#   no names: every task in evals/tasks/ (--cases: every legacy case in evals/cases/)
#   --repeat N  trials per task (default 1); --jobs J parallel trials (default 4)
#   --root DIR  take KERNEL.md and the skills from another checkout of this repo
#               (e.g. `git worktree add` of HEAD) to compare a baseline
#   --graders   free: test each task's checks against its reference solution
#
# A task is evals/tasks/<name>/ (a legacy case, evals/cases/<name>/, has the same files):
#   task.md     what it covers and why: `Covers:`, `Kind: capability|regression`, `Turns: N`
#   prompt.md   the first user message (required)
#   user.md     a simulated user's hidden situation; makes the task multi-turn (optional)
#   fixture/    files copied into a fresh git repo (optional)
#   work/       task state copied into the run's $WORK, outside the repo (optional)
#   setup.sh    run in the repo after the first commit (optional)
#   checks.sh   named outcome checks: `check "<label>" <command ...>` (lib.sh helpers)
#   check.sh    legacy: checks that stop at the first failure
#   rubric.md   numbered dimensions, each graded PASS / FAIL / UNKNOWN by a model
#   reference/apply.sh   produces a correct outcome in the repo, for --graders
#
# The run repo gets the kernel the way a real repo does: KERNEL.md imported
# from AGENTS.md, and each skill linked into .claude/skills. Only project
# settings load, so user-level skills and memory don't interfere.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EVALS="$ROOT/evals"
REPEAT=1
SKILLS_ROOT="${SKILLS_ROOT:-$ROOT}"
JOBS=4
MODE=run
SET=tasks
while [ $# -gt 0 ]; do
  case "$1" in
    --repeat) REPEAT="$2"; shift 2 ;;
    --jobs) JOBS="$2"; shift 2 ;;
    --root) SKILLS_ROOT="$(cd "$2" && pwd)"; shift 2 ;;
    --cases) SET=cases; shift ;;
    --graders) MODE=graders; shift ;;
    --one) ONE=1; shift ;;
    *) break ;;
  esac
done

src_of() {
  if [ -d "$EVALS/tasks/$1" ]; then echo "$EVALS/tasks/$1"
  elif [ -d "$EVALS/cases/$1" ]; then echo "$EVALS/cases/$1"
  else echo "no task or case named '$1'" >&2; return 1; fi
}

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

# Claude Code keeps per-project state for every directory a session runs in
# (~/.claude/projects/<path>, /private/tmp/claude-<uid>/<path>, with every
# non-alphanumeric character of the path turned into "-"). Each eval run makes
# fresh directories, so remove their state afterwards; the transcript is
# already saved in the run folder.
forget_sessions_in() {
  local real enc
  real="$(cd "$1" 2>/dev/null && pwd -P)" || return 0
  case "$real/" in "$(cd "$OUT" && pwd -P)/"?*) ;; *) return 0 ;; esac
  enc="$(printf %s "$real" | sed 's|[^A-Za-z0-9]|-|g')"
  rm -rf "$HOME/.claude/projects/$enc" "/private/tmp/claude-$(id -u)/$enc"
}

# prepare <src> <dir>: build the run repo and state folder; sets repo, SAM_STACK_WORK, WORK.
prepare() {
  local src="$1" dir="$2" v
  repo="$dir/repo"
  mkdir -p "$repo"
  # Per-run state folder; the project name is the repo's directory name.
  export SAM_STACK_WORK="$dir/work" WORK="$dir/work/repo"
  mkdir -p "$WORK"
  [ -d "$src/fixture" ] && cp -R "$src/fixture/." "$repo/"
  [ -d "$src/work" ] && cp -R "$src/work/." "$WORK/"
  cp "$SKILLS_ROOT/kernel/KERNEL.md" "$repo/KERNEL.md"
  echo "@KERNEL.md" > "$repo/AGENTS.md"
  mkdir -p "$repo/.claude/skills"
  for v in "$SKILLS_ROOT"/kernel/*/ "$SKILLS_ROOT"/workflows/*/; do
    [ -f "$v/SKILL.md" ] && ln -sn "$v" "$repo/.claude/skills/$(basename "$v")"
  done
  (cd "$repo" && git init -q && git add -A && git commit -qm "fixture")
  [ -f "$src/setup.sh" ] && (cd "$repo" && ROOT="$ROOT" bash "$src/setup.sh")
  return 0
}

# run_checks <src> <transcript> <out-file>: named checks (or legacy check.sh), one line each.
run_checks() {
  local src="$1" transcript="$2" out="$3"
  : > "$out"
  if [ -f "$src/checks.sh" ]; then
    (cd "$repo" && TRANSCRIPT="$transcript" bash -c "source '$EVALS/lib.sh'; source '$src/checks.sh'") >> "$out" 2>&1
  elif [ -f "$src/check.sh" ]; then
    local note
    if note="$(cd "$repo" && TRANSCRIPT="$transcript" bash -c "source '$EVALS/lib.sh'; source '$src/check.sh'" 2>&1)"
    then echo "PASS legacy check.sh" >> "$out"; else echo "FAIL legacy check.sh: ${note#FAIL: }" >> "$out"; fi
  fi
  [ -e "$repo/work" ] && echo "FAIL task state stays out of the repo: repo has a work/ folder" >> "$out"
  return 0
}

# agent_turn <dir> <message> [session-id]: one headless turn, appended to the transcript.
agent_turn() {
  local dir="$1" msg="$2" sid="${3:-}" n
  n=$(ls "$dir"/turn-*.jsonl 2>/dev/null | wc -l | tr -d ' '); n=$((n + 1))
  (cd "$repo" && claude -p "$msg" ${sid:+--resume "$sid"} \
      --setting-sources project \
      --output-format stream-json --verbose \
      --permission-mode acceptEdits \
      --add-dir "$SKILLS_ROOT/kernel" --add-dir "$SKILLS_ROOT/workflows" --add-dir "$SAM_STACK_WORK" \
      --allowedTools "Bash(git:*) Bash(python3:*) Bash(ls:*) Bash(cat:*) Bash(mkdir:*) Bash(grep:*) Bash(find:*) Bash(chmod:*) Bash(bash:*) Bash(sh:*) Bash(evals/:*)" \
      --disallowedTools "AskUserQuestion" \
      > "$dir/turn-$n.jsonl" 2>> "$dir/stderr.log")
  cat "$dir/turn-$n.jsonl" >> "$dir/transcript.jsonl"
  printf 'AGENT:\n%s\n\n' "$(jq -r 'select(.type=="result") | .result // empty' "$dir/turn-$n.jsonl" | tail -1)" >> "$dir/conversation.md"
  jq -r 'select(.type=="result") | .session_id // empty' "$dir/turn-$n.jsonl" | tail -1
}

# user_reply <src> <dir>: the simulated user's next message, or <<END>>.
user_reply() {
  local src="$1" dir="$2"
  mkdir -p "$dir/user"
  (cd "$dir/user" && claude -p --setting-sources project "You are role-playing the user in a conversation with a coding agent. Your hidden situation and preferences are below. Reply to the agent's latest message as this user would: answer only what it asked, briefly and in your own words, using only what your situation says. If it asks something your situation doesn't cover, say you don't mind and let it choose. If it asks for a fact it could look up in the repo, say so. If its latest message asks you nothing (it has finished, or is only reporting), reply with exactly <<END>> and nothing else.

<situation>
$(cat "$src/user.md")
</situation>

<conversation>
$(cat "$dir/conversation.md")
</conversation>") 2>> "$dir/stderr.log"
}

# grade <src> <dir>: one model grader, one line per rubric dimension.
grade() {
  local src="$1" dir="$2" work
  work="$(cd "$SAM_STACK_WORK" && find . -type f -exec sh -c 'echo "=== $1"; cat "$1"' _ {} \;)"
  mkdir -p "$dir/grader"
  (cd "$dir/grader" && claude -p --setting-sources project "You are grading one run of an agent. Judge each numbered rubric dimension on its own, only from the transcript and files below. Use UNKNOWN when they don't show enough to decide; don't guess. Output exactly one line per dimension and nothing else, in this form:
<n>: PASS|FAIL|UNKNOWN — <one line of evidence quoted from the transcript or files>

<rubric>
$(cat "$src/rubric.md")
</rubric>

<transcript>
$(cat "$dir/digest.txt")
</transcript>

<work-files>
$work
</work-files>") > "$dir/grade.md" 2>&1
}

run_one() {
  local name="$1" rep="$2" src dir sid reply turns i
  src="$(src_of "$name")" || return 1
  dir="$OUT/$name/r$rep"
  prepare "$src" "$dir"

  turns="$(sed -n 's/^Turns: *\([0-9]*\).*/\1/p' "$src/task.md" 2>/dev/null)"; turns="${turns:-8}"
  sid="$(agent_turn "$dir" "$(cat "$src/prompt.md")")"
  if [ -f "$src/user.md" ]; then
    for i in $(seq 2 "$turns"); do
      reply="$(user_reply "$src" "$dir")"
      printf 'USER:\n%s\n\n' "$reply" >> "$dir/conversation.md"
      grep -qx '<<END>>' <<<"$reply" && break
      [ -n "$sid" ] || break
      sid="$(agent_turn "$dir" "$reply" "$sid")"
    done
  fi
  digest "$dir/transcript.jsonl" > "$dir/digest.txt"

  run_checks "$src" "$dir/transcript.jsonl" "$dir/checks.txt"
  [ -f "$src/rubric.md" ] && grade "$src" "$dir"

  local cp cf rp rf ru result=PASS
  cp=$(grep -c '^PASS' "$dir/checks.txt"); cf=$(grep -c '^FAIL' "$dir/checks.txt")
  rp=0 rf=0 ru=0
  if [ -f "$dir/grade.md" ]; then
    rp=$(grep -cE '^[0-9]+: PASS' "$dir/grade.md"); rf=$(grep -cE '^[0-9]+: FAIL' "$dir/grade.md")
    ru=$(grep -cE '^[0-9]+: UNKNOWN' "$dir/grade.md")
    [ $((rp + rf + ru)) -gt 0 ] || { rf=1; echo "0: FAIL — grader output unreadable" >> "$dir/grade.md"; }
  fi
  [ "$cf" -eq 0 ] && [ "$rf" -eq 0 ] && [ "$ru" -eq 0 ] || result=FAIL

  forget_sessions_in "$repo"; forget_sessions_in "$dir/grader"; forget_sessions_in "$dir/user"
  printf "%-26s r%-3s %s  checks %s/%s  rubric %s/%s%s  turns %s\n" "$name" "$rep" "$result" \
    "$cp" $((cp + cf)) "$rp" $((rp + rf + ru)) "$([ "$ru" -gt 0 ] && echo " ($ru unknown)")" \
    "$(ls "$dir"/turn-*.jsonl | wc -l | tr -d ' ')" | tee -a "$OUT/results.txt"
  grep -h '^FAIL' "$dir/checks.txt" | sed 's/^/    check  /'
  grep -hE '^[0-9]+: (FAIL|UNKNOWN)' "$dir/grade.md" 2>/dev/null | sed 's/^/    rubric /'
  return 0
}

# graders_one <task>: checks must fail on the untouched fixture and pass on the reference.
graders_one() {
  local name="$1" src dir before after ok=PASS
  src="$(src_of "$name")" || return 1
  [ -x "$src/reference/apply.sh" ] || [ -f "$src/reference/apply.sh" ] || { printf "%-26s SKIP  no reference/apply.sh\n" "$name"; return 0; }
  dir="$OUT/graders/$name"
  prepare "$src" "$dir"
  run_checks "$src" /dev/null "$dir/untouched.txt"
  (cd "$repo" && ROOT="$ROOT" bash "$src/reference/apply.sh")
  run_checks "$src" /dev/null "$dir/reference.txt"
  before=$(grep -c '^FAIL' "$dir/untouched.txt"); after=$(grep -c '^FAIL' "$dir/reference.txt")
  [ "$before" -gt 0 ] && [ "$after" -eq 0 ] || ok=FAIL
  printf "%-26s %s  untouched fails %s/%s  reference passes %s/%s\n" "$name" "$ok" "$before" \
    "$(grep -c '' "$dir/untouched.txt")" "$(grep -c '^PASS' "$dir/reference.txt")" "$(grep -c '' "$dir/reference.txt")" \
    | tee -a "$OUT/results.txt"
  grep -h '^FAIL' "$dir/reference.txt" | sed 's/^/    reference /'
  return 0
}

# Worker mode: xargs re-invokes this script once per (name, rep).
if [ "${ONE:-}" = 1 ]; then run_one "$1" "$2"; exit 0; fi

# Outside this repo: a run repo nested in it would inherit its AGENTS.md.
OUT="${EVALS_OUT:-${XDG_CACHE_HOME:-$HOME/.cache}/sam_stack-evals}/$(date +%Y%m%d-%H%M%S)-$$"
mkdir -p "$OUT"
export OUT SKILLS_ROOT

names=("$@")
[ ${#names[@]} -eq 0 ] && names=($(ls "$EVALS/$SET"))

if [ "$MODE" = graders ]; then
  for n in "${names[@]}"; do graders_one "$n"; done
  echo "Runs: $OUT"
  ! grep -q " FAIL " "$OUT/results.txt"
  exit
fi

for n in "${names[@]}"; do
  for r in $(seq 1 "$REPEAT"); do echo "$n $r"; done
done | xargs -n 2 -P "$JOBS" "$0" --one

echo
echo "Summary (passing trials / trials):"
for n in "${names[@]}"; do
  printf "  %-26s %s/%s\n" "$n" "$(grep -c "^$n .* PASS " "$OUT/results.txt")" "$REPEAT"
done
echo "Runs: $OUT"
! grep -q " FAIL " "$OUT/results.txt"
