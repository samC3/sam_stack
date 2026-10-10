invoked adversarial-review
# A workflow step that names a verb runs that verb's skill, not an inline imitation of it.
for v in ground judge prove distill; do invoked "$v"; done
has "work/review-*/verdicts.md" "^## J[0-9]"
has "work/review-*/proofs.md" "^## P[0-9]"
has "work/review-*/review.md" "[Qq]uot|[Cc]omma"
git diff --quiet reviewed -- export.py test_export.py work/csv-export || fail "the reviewer edited the work under review"
