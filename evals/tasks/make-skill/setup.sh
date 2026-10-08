# Give the run repo the free tooling only. No run.sh, so no nested paid runs.
mkdir -p evals/cases
cp "$ROOT/evals/lint.sh" "$ROOT/evals/lib.sh" evals/
git add -A && git commit -qm "Add eval tooling"
