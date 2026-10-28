T="$SAM_STACK_WORK/todo"
TODAY="$T/days/$(date +%F).md"
ranked() { grep -E '^[0-9]+\. ' "$T/now.md" 2>/dev/null; }
three_to_five_ranked() { n="$(ranked | wc -l)"; [ "$n" -ge 3 ] && [ "$n" -le 5 ] || fail "now.md has $n ranked lines"; }
each_ranked_is_open() {
  ranked >/dev/null || fail "no ranked lines"
  ranked | sed -E 's/^[0-9]+\. +//; s/  .*//' | while read -r item; do
    grep -Eq "^TODO +$item  " "$TODAY" "$T/backlog.md" || fail "'$item' isn't an open item"
  done
}
top_three_follow_the_user() {
  ranked | sed -n 1p | grep -q "Fix the checkout bug" || fail "first isn't the checkout bug"
  ranked | sed -n 2p | grep -q "Renew the passport" || fail "second isn't the passport"
  ranked | sed -n 3p | grep -q "Write the Q3 roadmap" || fail "third isn't the roadmap"
}

check "now.md ranks 3 to 5 items"                 three_to_five_ranked
check "every ranked item is an open item"         each_ranked_is_open
check "the ranking follows the user's priorities" top_three_follow_the_user
check "decisions logged beside the list"          has "$T/decisions.md" "^## D[0-9]"
check "no decisions logged in the project folder" test ! -e "$WORK/todo/decisions.md"

check "invoked todo"                              invoked todo
check "invoked ask"                               invoked ask
