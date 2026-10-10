ROOT_COMMIT="$(git rev-list --max-parents=0 HEAD)"

# lint_with_stop <text>: lint a copy of the repo whose notes skill has "Stop:<tab><text>".
lint_with_stop() {
  local t s; t="$(mktemp -d)"; cp -R kernel evals "$t/"
  python3 -c "import re,sys,pathlib;p=pathlib.Path(sys.argv[1]);p.write_text(re.sub(r'(?m)^Stop:.*$','Stop:\t'+sys.argv[2],p.read_text()))" \
    "$t/kernel/notes/SKILL.md" "$1"
  python3 evals/lint.py "$t" >/dev/null 2>&1; s=$?; rm -rf "$t"; return $s
}
rejects_tab_only()   { ! lint_with_stop ""; }
accepts_tab_text()   { lint_with_stop "the entry is written"; }
new_lint_case()      { ! git diff --quiet "$ROOT_COMMIT" -- evals/lint_test.py && python3 evals/lint_test.py >/dev/null; }
unchanged()          { git diff --quiet "$ROOT_COMMIT" -- "$@" && [ -z "$(git ls-files --others --exclude-standard -- "$@")" ]; }

# Lesson 1: a mechanism in lint, both directions, with a regression fixture.
check "lint rejects a tab-only contract field"          rejects_tab_only
check "lint still accepts a tab followed by text"       accepts_tab_text
check "a lint regression case was added"                new_lint_case
check "lint and its tests pass on the repo"             sh -c 'python3 evals/lint.py . && python3 evals/lint_test.py'
# Lesson 2: below the evidence bar, so a candidate and nothing else.
check "the single incident is logged as a candidate"   has "$SAM_STACK_WORK/lessons/candidates.md" "[Bb]rief|[Ii]nvent|[Rr]efund"
check "the brief skill and its task are unchanged"      unchanged kernel/brief evals/tasks/brief-keeps-gaps
# Lesson 3: a kernel-wide rule, proposed and waiting for sign-off.
check "neither kernel was edited"                       unchanged KERNEL.md kernel/KERNEL.md
check "a lesson awaits sign-off"                        has "$WORK/*/lessons.md" "[Aa]waiting sign-off"
check "notes skill restored after the repro"            unchanged kernel/notes
