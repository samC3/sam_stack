#!/usr/bin/env bash
# Regression tests for evals/lint.sh. Free and deterministic.
#
# Each fixture is evals/lint-fixtures/<name>/: a miniature repo (kernel/,
# workflows/, evals/cases/) plus an `expect` file. The first line of `expect`
# is either `ok` (lint must pass) or a string that must appear in lint's
# output when it fails.
set -uo pipefail

EVALS="$(cd "$(dirname "$0")" && pwd)"
failed=0

for f in "$EVALS"/lint-fixtures/*/; do
  name="$(basename "$f")"
  want="$(head -1 "$f/expect")"
  out="$(bash "$EVALS/lint.sh" "$f" 2>&1)"; status=$?
  if [ "$want" = "ok" ]; then
    [ $status -eq 0 ] && r=PASS || r=FAIL
  else
    [ $status -ne 0 ] && grep -qF -- "$want" <<<"$out" && r=PASS || r=FAIL
  fi
  printf "%-32s %s\n" "$name" "$r"
  [ "$r" = PASS ] || { failed=$((failed + 1)); sed 's/^/    /' <<<"$out"; }
done

[ $failed -eq 0 ] && echo "lint-test: ok" || { echo "lint-test: $failed failed"; exit 1; }
