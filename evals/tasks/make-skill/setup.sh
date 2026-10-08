# Give the run repo the free tooling only: the linter, not the runner, so no nested paid runs.
mkdir -p evals/tasks
cp "$ROOT/evals/lint.py" "$ROOT/evals/lib.sh" evals/
git add -A && git commit -qm "Add eval tooling"
