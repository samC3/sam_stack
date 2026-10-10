# Helpers for a task's checks.sh. Run with cwd = the run repo; $WORK and
# $SAM_STACK_WORK point at the run's task state, $TRANSCRIPT at its transcript.

fail() { echo "FAIL: $*"; exit 1; }

# has <file-glob> <extended-regex>: some file matching the glob contains the pattern.
has() {
  local f
  for f in $1; do [ -f "$f" ] && grep -Eq "$2" "$f" && return 0; done
  fail "no file matching $1 contains /$2/"
}

# check <label> <command ...>: run one named outcome check and record it as a
# PASS or FAIL line without stopping, so a task gets partial credit.
check() {
  local label="$1" out; shift
  if out="$( ( "$@" ) 2>&1 )"; then echo "PASS $label"
  else echo "FAIL $label: ${out#FAIL: }" | head -1; fi
}

# invoked <skill>: the agent called the Skill tool for <skill> during the run,
# so that skill's procedure was in context (KERNEL.md Layers).
invoked() {
  python3 - "$TRANSCRIPT" "$1" <<'PY' || fail "never invoked $1"
import json, sys
for line in open(sys.argv[1]):
    try:
        event = json.loads(line)
    except ValueError:
        continue
    for part in (event.get("message") or {}).get("content", []) if event.get("type") == "assistant" else []:
        if isinstance(part, dict) and part.get("name") == "Skill" and (part.get("input") or {}).get("skill") == sys.argv[2]:
            sys.exit(0)
sys.exit(1)
PY
}
