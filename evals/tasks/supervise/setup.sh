# The parser child's worktree and branch, as `session.py start --edit` would have
# left them, with the child's commit: it accepts unpadded dates, so one test fails.
git worktree add -q .worktrees/parser -b supervise/parser
cat > .worktrees/parser/dates.py <<'PY'
from datetime import date


def parse_date(text):
    """Parse a strict ISO date, YYYY-MM-DD. Raise ValueError for anything else."""
    year, month, day = map(int, text.split("-"))
    return date(year, month, day)
PY
git -C .worktrees/parser commit -qam "Implement parse_date"
git rev-parse supervise/parser > .git/parser-tip
# The children wrote their returns after their last message, so they aren't stale.
touch "$WORK"/dates-work/sessions/*/return.md
