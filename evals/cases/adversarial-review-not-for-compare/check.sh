# Comparing options is judge's job, not a goal-based review.
invoked judge
if jq -e 'select(.type=="assistant") | .message.content[]?
  | select(.type=="tool_use" and .name=="Skill" and .input.skill=="adversarial-review")' \
  "$TRANSCRIPT" >/dev/null; then fail "adversarial-review fired on a comparison"; fi
