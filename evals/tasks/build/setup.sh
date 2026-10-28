git tag base
git rev-parse --abbrev-ref HEAD > "$SAM_STACK_WORK/../base-branch"
cp -R "$WORK/slugify/tickets" "$SAM_STACK_WORK/../shape-state"
