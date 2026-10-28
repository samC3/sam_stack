T="$SAM_STACK_WORK/todo"
cat > "$T/now.md" <<'MD'
# Now
1. Fix the checkout bug  [repo]
2. Renew the passport  [_global]
3. Write the Q3 roadmap  [repo]
MD
cat > "$T/decisions.md" <<'MD'
# Decisions: todo

## D1: The checkout bug comes first, then the passport, then the Q3 roadmap
Chose: that order. Because: customers are hitting the bug; the passport is due soon
Rejected: roadmap first (less urgent)
MD
