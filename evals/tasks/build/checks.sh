S="$WORK/slugify"

check "slugify meets the spec"               python3 -c "from textkit import slugify as s; assert s('  Hello, World! ') == 'hello-world'; assert s('!!!') == ''; assert s('A--B__c') == 'a-b-c'; assert s('Café 2') == 'caf-2'"
check "existing tests still pass"            python3 -m unittest -q
check "tests for slugify were added"         has "test_*.py" "slugify"
check "no comments or docstrings added"      "$EVALS_BIN" comments base
check "existing comments left alone"         test -z "$(git diff base -- textkit.py test_textkit.py | grep -E '^-[[:space:]]*(#|""")')"
check "ticket 02 not built (waits on O1)"    python3 -c "import inspect, textkit; assert len(inspect.signature(textkit.slugify).parameters) == 1"
check "work is on a new branch"              test "$(git rev-parse --abbrev-ref HEAD)" != "$(cat "$SAM_STACK_WORK/../base-branch")"
check "ticket 01 committed"                  test -n "$(git log base..HEAD --format=%s | grep -Ei '01|slugify')"
check "nothing left uncommitted"             test -z "$(git status --porcelain -- '*.py')"
check "a red run was recorded"               has "$S/proofs.md" "^Verdict: fails"
check "a green run was recorded"             has "$S/proofs.md" "^Verdict: holds"
check "the diff was judged"                  has "$S/verdicts.md" "^## J[0-9]"
check "pr.md written"                        test -f "$S/pr.md"
check "shape's files untouched"              diff -r "$SAM_STACK_WORK/../shape-state" "$S/tickets"

# Every covered skill ran as a skill, not inline (KERNEL.md Layers).
check "invoked build"                        invoked build
check "invoked ground"                       invoked ground
check "invoked prove"                        invoked prove
check "invoked judge"                        invoked judge
check "invoked distill"                      invoked distill
