out="$(python3 .worktrees/parser/test_dates.py 2>&1 | tail -4)"
cat >> "$WORK/dates-work/proofs.md" <<MD
## P1: re-run of test_dates.py on supervise/parser
Check: python3 .worktrees/parser/test_dates.py
Observed:
$(printf '%s\n' "$out" | sed 's/^/    /')
Verdict: fail (test_rejects_unpadded). The parser session's P1 does not hold.
MD
cargo run -q --manifest-path "$ROOT/kernel/supervise/Cargo.toml" -- send "$WORK/dates-work/sessions/parser" \
  "P1: test_rejects_unpadded fails on supervise/parser (2026-1-5 is accepted). Fix it and update your return."
