T="$SAM_STACK_WORK/todo"
D="$(date +%F)"
mkdir -p "$T/details"
cat > "$T/days/2026-01-05.md" <<MD
# 2026-01-05
MOVED    Write the release notes  [repo] (to $D)
MOVED    Tidy the CI config  [repo] (to backlog)
MOVED    Fix the flaky login test  [repo] → flaky-login (to $D)
DONE     Book the venue  [_global]
MD
cat > "$T/backlog.md" <<'MD'
TODO     Tidy the CI config  [repo]
MD
cat > "$T/days/$D.md" <<MD
# $D
TODO     Write the release notes  [repo]  [details](../details/write-the-release-notes.md)
DONE     Fix the flaky login test  [repo] → flaky-login
TODO     Draft the Q3 roadmap  [repo]
TODO     Upgrade the database driver  [repo]
MD
cat > "$T/details/write-the-release-notes.md" <<'MD'
# Write the release notes
- include the migration steps
MD
