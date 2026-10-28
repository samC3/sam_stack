T="$SAM_STACK_WORK/todo"
TODAY="$T/days/$(date +%F).md"
OLD="$T/days/2026-01-05.md"
pulled_left_backlog() { ! grep -q "Upgrade the database driver" "$T/backlog.md"; }

check "new item added to today's file with project tag" has "$TODAY" "^TODO +Draft the Q3 roadmap +\[repo\]"
check "carried item marked moved to today"              has "$OLD" "^MOVED +Write the release notes.*\(to $(date +%F)\)"
check "backlogged item marked moved to backlog"         has "$OLD" "^MOVED +Tidy the CI config.*\(to backlog\)"
check "backlogged item appended to backlog.md"          has "$T/backlog.md" "^TODO +Tidy the CI config +\[repo\]"
check "pulled item removed from backlog.md"             pulled_left_backlog
check "pulled item is a TODO in today's file"           has "$TODAY" "^TODO +Upgrade the database driver +\[repo\]"
check "note written to the item's details file"         has "$T/details/write-the-release-notes.md" "migration steps"
check "carried item links to its details file"          has "$TODAY" "^TODO +Write the release notes.*\[details\]\(\.\./details/write-the-release-notes\.md\)"
check "archived slug's item is done in today's file"    has "$TODAY" "^DONE +Fix the flaky login test.*→ flaky-login"

check "invoked todo"                                    invoked todo
