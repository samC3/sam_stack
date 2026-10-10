# Remember where the build started, so checks can diff against it.
git tag base
git rev-parse --abbrev-ref HEAD > "$SAM_STACK_WORK/../base-branch"
# A copy of shape's tickets, to show build didn't edit them.
cp -R "$WORK/slugify/tickets" "$SAM_STACK_WORK/../shape-state"
