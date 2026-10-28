fail() { echo "FAIL: $*"; exit 1; }

has() {
  local f
  for f in $1; do [ -f "$f" ] && grep -Eq "$2" "$f" && return 0; done
  fail "no file matching $1 contains /$2/"
}

check() {
  local label="$1" out; shift
  if out="$( ( "$@" ) 2>&1 )"; then echo "PASS $label"
  else echo "FAIL $label: ${out#FAIL: }" | head -1; fi
}

invoked() {
  "$EVALS_BIN" invoked "$TRANSCRIPT" "$1" || fail "never invoked $1"
}
