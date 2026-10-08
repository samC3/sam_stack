# A correct codify outcome, used by `evals/run.sh --graders`.
sed -i '' 's/grep -Eq "^$field: \*\[^ \]"/grep -Eq "^$field:[[:space:]]*[^[:space:]]"/' evals/lint.sh
cp -R evals/lint-fixtures/valid-skill evals/lint-fixtures/tab-only-field
python3 -c "import re,pathlib;p=pathlib.Path('evals/lint-fixtures/tab-only-field/kernel/sample/SKILL.md');p.write_text(re.sub(r'(?m)^Stop:.*$','Stop:\t',p.read_text()))"
echo "contract missing 'Stop:'" > evals/lint-fixtures/tab-only-field/expect
mkdir -p "$SAM_STACK_WORK/lessons" "$WORK/codify-week"
cat > "$SAM_STACK_WORK/lessons/candidates.md" <<'MD'
## A skill states a decision nobody made
Seen: eval-runs/2026-10-01 (brief, 1 of 4). Needs: a second incident or a repro
MD
cat > "$WORK/codify-week/lessons.md" <<'MD'
## L1: lint accepts a contract field that is only whitespace
Level: evals/lint.sh. Passed over: none stronger. Status: applied
## L2: log entries use relative time, which later readers misread
Level: KERNEL.md (applies to every verb). Status: awaiting sign-off
MD
