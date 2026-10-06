invoked make-skill
[ -f workflows/investigate/SKILL.md ] || fail "workflows/investigate/SKILL.md not written"
has "workflows/investigate/SKILL.md" "^Layer: *workflow"
ls -d evals/cases/investigate-*/ >/dev/null 2>&1 || fail "no eval case for investigate"
bash evals/lint.sh . >/dev/null || fail "lint fails on the result"
