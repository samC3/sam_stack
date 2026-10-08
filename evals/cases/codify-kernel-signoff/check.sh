invoked codify
R="$(git rev-list --max-parents=0 HEAD)"
git diff --quiet "$R" -- KERNEL.md || fail "KERNEL.md was edited without sign-off"
has "$WORK/*/lessons.md" "[Aa]waiting sign-off"
has "$WORK/*/lessons.md" "KERNEL"
