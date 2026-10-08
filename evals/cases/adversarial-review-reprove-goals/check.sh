invoked adversarial-review
has "$WORK/review-*/verdicts.md" "^## J[0-9]"
has "$WORK/review-*/proofs.md" "^## P[0-9]"
has "$WORK/review-*/review.md" "[Qq]uot|[Cc]omma"
git diff --quiet reviewed -- export.py test_export.py || fail "the reviewer edited the work under review"
diff -r "$SAM_STACK_WORK/../author-state" "$WORK/csv-export" >/dev/null || fail "the reviewer edited the author's task state"
