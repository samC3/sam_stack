T="$SAM_STACK_WORK/todo"
mkdir -p "$T/days" "$WORK/_archive/flaky-login"
cat > "$T/days/2026-01-05.md" <<'MD'
# 2026-01-05
TODO     Write the release notes  [repo]
TODO     Tidy the CI config  [repo]
TODO     Fix the flaky login test  [repo] → flaky-login
DONE     Book the venue  [_global]
MD
cat > "$T/backlog.md" <<'MD'
TODO     Upgrade the database driver  [repo]
MD
