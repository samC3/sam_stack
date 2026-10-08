# A correct codify outcome, used by `evals/run.py --graders`.
sed -i '' 's/value.lstrip(" ")/value.strip()/' evals/lint.py
python3 - <<'PY'
from pathlib import Path
p = Path("evals/lint_test.py"); s = p.read_text()
anchor = '    ("tab-aligned-field",'
case = '    ("tab-only-field",         sub(SKILL, STOP, "Stop:\\t"),                                     "\'Stop\' is missing or empty"),\n'
p.write_text(s.replace(anchor, case + anchor, 1))
PY
mkdir -p "$SAM_STACK_WORK/lessons" "$WORK/codify-week"
cat > "$SAM_STACK_WORK/lessons/candidates.md" <<'MD'
## A skill states a decision nobody made
Seen: eval-runs/2026-10-01 (brief, 1 of 4). Needs: a second incident or a repro
MD
cat > "$WORK/codify-week/lessons.md" <<'MD'
## L1: lint accepts a contract field that is only whitespace
Level: evals/lint.py. Passed over: none stronger. Status: applied
## L2: log entries use relative time, which later readers misread
Level: KERNEL.md (applies to every verb). Status: awaiting sign-off
MD
