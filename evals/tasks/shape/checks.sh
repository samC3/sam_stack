S="$(ls "$WORK"/*/spec.md 2>/dev/null | head -1)"
T="$(dirname "${S:-$WORK/none/x}")/tickets"
every_ticket_has_verify() { ls "$T"/*.md >/dev/null 2>&1 || fail "no tickets"; for f in "$T"/*.md; do grep -q "Verify" "$f" || fail "$f has no Verify"; done; }
check "spec written"                    test -n "$S"
check "tickets written, each with Verify" every_ticket_has_verify
check "user's decisions logged"         has "$WORK/*/decisions.md" "^## D[0-9]"
check "an open question logged"         has "$WORK/*/decisions.md" "^## O[0-9]"
check "spec and tickets judged"         has "$WORK/*/verdicts.md" "^## J[0-9]"
check "repo facts logged with sources"  has "$WORK/*/findings.md" "\[fact: "
check "no code changed"                 test -z "$(git status --porcelain)"
