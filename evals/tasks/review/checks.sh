R="$(ls -d "$WORK"/review-*/ 2>/dev/null | head -1)"
check "review.md written"                    test -f "$R/review.md"
check "overall verdict is changes needed"    has "$R/review.md" "[Cc]hanges needed"
check "quoting criterion marked not met"     has "$R/review.md" "([Qq]uot|[Cc]omma).*[Nn]ot met|[Nn]ot met.*([Qq]uot|[Cc]omma)"
check "reviewer recorded its own proof"      has "$R/proofs.md" "^Observed"
check "goal verdicts recorded"               has "$R/verdicts.md" "^## J[0-9]"
check "work under review untouched"          git diff --quiet reviewed -- export.py test_export.py
check "author's task state untouched"        diff -r "$SAM_STACK_WORK/../author-state" "$WORK/csv-export"
