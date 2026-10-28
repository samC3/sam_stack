ROOT_COMMIT="$(git rev-list --max-parents=0 HEAD)"
TAB="$(printf '\t')"

lint_with_stop() {
  local t s; t="$(mktemp -d)"; cp -R kernel evals "$t/"
  sed -i '' "s/^Stop:.*\$/Stop:$TAB$1/" "$t/kernel/notes/SKILL.md"
  cargo run -q -p evals -- lint "$t" >/dev/null 2>&1; s=$?; rm -rf "$t"; return $s
}
rejects_tab_only()   { ! lint_with_stop ""; }
accepts_tab_text()   { lint_with_stop "the entry is written"; }
new_lint_case()      { ! git diff --quiet "$ROOT_COMMIT" -- evals/src/lint/tests.rs && cargo test -q -p evals >/dev/null 2>&1; }
unchanged()          { git diff --quiet "$ROOT_COMMIT" -- "$@" && [ -z "$(git ls-files --others --exclude-standard -- "$@")" ]; }

check "lint rejects a tab-only contract field"          rejects_tab_only
check "lint still accepts a tab followed by text"       accepts_tab_text
check "a lint regression case was added"                new_lint_case
check "lint and its tests pass on the repo"             sh -c 'cargo lint . >/dev/null && cargo test -q -p evals >/dev/null 2>&1'
check "the single incident is logged as a candidate"   has "$SAM_STACK_WORK/lessons/candidates.md" "[Bb]rief|[Ii]nvent|[Rr]efund"
check "the brief skill and its task are unchanged"      unchanged kernel/brief evals/tasks/brief-keeps-gaps
check "neither kernel was edited"                       unchanged KERNEL.md kernel/KERNEL.md
check "a lesson awaits sign-off"                        has "$WORK/*/lessons.md" "[Aa]waiting sign-off"
check "notes skill restored after the repro"            unchanged kernel/notes

check "invoked codify"                         invoked codify
check "invoked prove"                          invoked prove
