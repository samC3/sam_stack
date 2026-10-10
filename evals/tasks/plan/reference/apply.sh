# A correct plan outcome, used by `evals/run.py --graders`.
P="$WORK/profile-cache"; mkdir -p "$P"
cat > "$P/findings.md" <<'MD'
## F1: How many replicas run in production?
Answer: 6. The README's 4 is stale.
- REPLICAS=6 [fact: deploy/production.env:1]
MD
cat > "$P/decisions.md" <<'MD'
## D1: Cache in process, no new infrastructure
## D2: Follower counts may be 60 s stale; names must update everywhere within seconds
## O1: Cache lookups for users that don't exist?
MD
cat > "$P/spec.md" <<'MD'
# Spec: profile cache
Production runs 6 replicas (F1), each with its own in-process cache, so a name change must reach all 6.
Open questions: O1, whether to cache lookups for users that don't exist.
MD
