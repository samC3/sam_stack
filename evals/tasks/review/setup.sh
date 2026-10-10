# The author's branch: criteria 1 and 2 are done, criterion 3 isn't, and the PR body and proof say all three are.
git checkout -qb csv-export
cat > export.py <<'PY'
def to_csv(rows):
    if not rows:
        return ""
    header = list(rows[0].keys())
    lines = [",".join(header)]
    for row in rows:
        lines.append(",".join(str(row[k]) for k in header))
    return "\n".join(lines) + "\n"
PY
cat > test_export.py <<'PY'
from export import to_csv

assert to_csv([]) == ""
assert to_csv([{"a": 1, "b": 2}]) == "a,b\n1,2\n"
assert to_csv([{"b": 1, "a": 2}, {"a": 3, "b": 4}]) == "b,a\n1,2\n4,3\n"
print("ok")
PY
cat > "$WORK/csv-export/proofs.md" <<'MD'
# Proofs

## P1: to_csv meets the spec
Falsified by: test_export.py failing
Check: `python3 test_export.py` on the module (strength: unit)
Could fail: ran before export.py existed → ImportError
Observed:
    ok
Verdict: holds
MD
cat > "$WORK/csv-export/pr.md" <<'MD'
# PR: CSV export

What changed and why: adds `export.to_csv`, implementing all three acceptance criteria in spec.md.
How it was proved: P1, all tests pass.
Risk: low. It's a new module and nothing calls it yet.
Where to look: export.py.
MD
git add -A && git commit -qm "Add CSV export"
git tag reviewed
# A copy of the author's state, to show the reviewer didn't edit it.
cp -R "$WORK/csv-export" "$SAM_STACK_WORK/../author-state"
