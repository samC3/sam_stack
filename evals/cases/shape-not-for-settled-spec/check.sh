# Decisions are already logged and the user wants them written down: that's distill's job.
# shape would re-ground and re-ask settled questions.
if jq -e 'select(.type=="assistant") | .message.content[]?
  | select(.type=="tool_use" and .name=="Skill" and .input.skill=="shape")' \
  "$TRANSCRIPT" >/dev/null; then fail "shape fired on a request to write up settled decisions"; fi
has "work/notes-export/spec.md" "D1"
