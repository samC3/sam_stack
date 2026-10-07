invoked adversarial-review
has "work/review-*/verdicts.md" "^## J[0-9]"
has "work/review-*/proofs.md" "^## P[0-9]"
has "work/review-*/review.md" "[Qq]uot|[Cc]omma"
git diff --quiet reviewed -- export.py test_export.py work/csv-export || fail "the reviewer edited the work under review"
