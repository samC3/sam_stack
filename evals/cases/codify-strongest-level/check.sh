invoked codify
R="$(git rev-list --max-parents=0 HEAD)"
git diff --quiet "$R" -- KERNEL.md kernel/ || fail "KERNEL.md or kernel/ changed: wording instead of a mechanism, or the repro was not restored"

T="$(mktemp -d)"; cp -R kernel evals "$T/"
python3 - "$T/kernel/notes/SKILL.md" <<'PY'
import re, sys, pathlib
p = pathlib.Path(sys.argv[1]); s = p.read_text()
p.with_suffix(".ok").write_text(s)
p.write_text(re.sub(r'(?m)^Stop:.*$', 'Stop:\t', s))
PY
out="$(bash evals/lint.sh "$T" 2>&1)" && fail "lint still accepts a tab-only Stop: field"
grep -q "Stop" <<<"$out" || fail "lint fails, but not on the Stop field: $out"
python3 -c "import re,sys,pathlib;p=pathlib.Path(sys.argv[1]);p.write_text(re.sub(r'(?m)^Stop:.*$','Stop:\tthe entry is written',p.with_suffix('.ok').read_text()))" "$T/kernel/notes/SKILL.md"
rm -f "$T/kernel/notes/SKILL.ok"
bash evals/lint.sh "$T" >/dev/null 2>&1 || fail "lint now rejects a tab followed by text (overfitted)"
bash evals/lint.sh . >/dev/null 2>&1 || fail "lint fails on the repo itself"

{ git diff --name-only "$R"; git ls-files --others --exclude-standard; } \
  | grep -Ev '^(evals/lint\.sh|KERNEL\.md|AGENTS\.md)' | grep -q . \
  || fail "no regression check added beside the lint fix"
has "$WORK/*/lessons.md" "^## L[0-9]"
