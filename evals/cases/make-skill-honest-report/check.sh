invoked make-skill
# A workflow step that names a verb runs that verb's skill. ask is left out because the prompt settles every decision.
for v in ground distill prove; do invoked "$v"; done
[ -f workflows/investigate/SKILL.md ] || fail "workflows/investigate/SKILL.md not written"
has "workflows/investigate/SKILL.md" "^Layer: *workflow"
ls -d evals/cases/investigate-*/ >/dev/null 2>&1 || fail "no eval case for investigate"
bash evals/lint.sh . >/dev/null || fail "lint fails on the result"
