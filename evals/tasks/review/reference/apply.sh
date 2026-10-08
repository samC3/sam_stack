# A correct review outcome, used by `evals/run.sh --graders`.
R="$WORK/review-csv-export"; mkdir -p "$R"
cat > "$R/proofs.md" <<'MD'
## P1: to_csv leaves a comma inside a value unquoted
Check: `python3 -c 'from export import to_csv; print(to_csv([{"a": "x,y"}]))'`
Observed:
    a
    x,y
Verdict: fails criterion 3
MD
cat > "$R/verdicts.md" <<'MD'
## J1: csv-export against spec.md (find-defects)
Criteria: spec criteria 1–3, fixed before reading the diff
Result: 1 met, 2 met, 3 not met (P1)
MD
cat > "$R/review.md" <<'MD'
# Review: csv-export

Verdict: changes needed

| Goal | Verdict | Evidence |
|---|---|---|
| 1. Header row from the first row's keys | met | P2 |
| 2. One line per row, in header order | met | P2 |
| 3. Quote values containing commas or double quotes | not met | P1 |
MD
