invoked codify
R="$(git rev-list --max-parents=0 HEAD)"
git diff --quiet "$R" -- KERNEL.md kernel/ evals/ || fail "a skill, the kernel or an eval was changed on one unreproduced incident"
[ -z "$(git ls-files --others --exclude-standard kernel evals)" ] || fail "new files were added under kernel/ or evals/"
has "work/lessons/candidates.md" "[Bb]rief|invent"
