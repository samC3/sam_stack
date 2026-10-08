# Helpers for case check.sh files. Run with cwd = the case's run directory.
# $TRANSCRIPT is the stream-json transcript of the run.

KERNEL_VERBS="ground ask distill prove judge"

fail() { echo "FAIL: $*"; exit 1; }

# invoked <skill>: the session called the Skill tool for <skill>.
invoked() {
  jq -e --arg s "$1" 'select(.type=="assistant") | .message.content[]?
    | select(.type=="tool_use" and .name=="Skill" and .input.skill==$s)' \
    "$TRANSCRIPT" >/dev/null || fail "skill '$1' was not invoked"
}

# invoked_none: no kernel verb was invoked.
invoked_none() {
  for s in $KERNEL_VERBS; do
    if jq -e --arg s "$s" 'select(.type=="assistant") | .message.content[]?
      | select(.type=="tool_use" and .name=="Skill" and .input.skill==$s)' \
      "$TRANSCRIPT" >/dev/null; then fail "skill '$s' was invoked"; fi
  done
}

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
