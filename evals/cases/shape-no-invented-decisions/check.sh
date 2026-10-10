# The request leaves real choices open (format, destination, which notes), so shape
# must stop at ask's first question: findings yes, spec and tickets not yet.
invoked shape
# A workflow step that names a verb runs that verb's skill, not an inline imitation of it.
invoked ground
invoked ask
has "work/*/findings.md" "^## F[0-9]"
for f in work/*/spec.md work/*/tickets/*.md; do
  [ -e "$f" ] && fail "wrote $f before the open choices were asked about"
done
git diff --quiet -- notes.py README.md || fail "edited the code while shaping"
