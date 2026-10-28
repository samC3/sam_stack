sed -i '' "s/value.trim_start_matches(' ')/value.trim()/" evals/src/lint.rs
awk '/^        \("tab-aligned-field",/ { print "        (\"tab-only-field\", sub(SKILL, STOP, \"Stop:\\t\"), \"'"'"'Stop'"'"' is missing or empty\")," } 1' \
  evals/src/lint/tests.rs > tests.rs.new && mv tests.rs.new evals/src/lint/tests.rs
mkdir -p "$SAM_STACK_WORK/lessons" "$WORK/codify-week"
cat > "$SAM_STACK_WORK/lessons/candidates.md" <<'MD'
## A skill states a decision nobody made
Seen: eval-runs/2026-10-01 (brief, 1 of 4). Needs: a second incident or a repro
MD
cat > "$WORK/codify-week/lessons.md" <<'MD'
## L1: lint accepts a contract field that is only whitespace
Level: evals/src/lint.rs. Passed over: none stronger. Status: applied
## L2: log entries use relative time, which later readers misread
Level: KERNEL.md (applies to every verb). Status: awaiting sign-off
MD
