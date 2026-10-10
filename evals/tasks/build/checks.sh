S="$WORK/slugify"

# no_new_comments: no line added since `base` in a .py file is a comment or a
# docstring, apart from directives a tool reads.
no_new_comments() {
  python3 - <<'PY' || fail "new comment or docstring added"
import ast, io, re, subprocess, sys, tokenize
diff = subprocess.run(["git", "diff", "-U0", "base", "--", "*.py"], capture_output=True, text=True).stdout
diff += "".join(f"+++ b/{f}\n@@ -0,0 +1,{sum(1 for _ in open(f))} @@\n" for f in subprocess.run(
    ["git", "ls-files", "--others", "--exclude-standard", "*.py"], capture_output=True, text=True).stdout.split())
added, path = {}, None
for line in diff.splitlines():
    if line.startswith("+++ "):
        path = line[6:] if line != "+++ /dev/null" else None
    m = re.match(r"@@ -\S+ \+(\d+)(?:,(\d+))? @@", line)
    if m and path:
        start, n = int(m[1]), int(m[2] if m[2] is not None else 1)
        added.setdefault(path, set()).update(range(start, start + n))
bad = []
for path, lines in added.items():
    src = open(path).read()
    for tok in tokenize.generate_tokens(io.StringIO(src).readline):
        if tok.type == tokenize.COMMENT and tok.start[0] in lines and not re.match(r"#!|#.*(noqa|type:|coding[:=])", tok.string):
            bad.append(f"{path}:{tok.start[0]} {tok.string}")
    for node in ast.walk(ast.parse(src)):
        if isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)) and ast.get_docstring(node) is not None:
            if node.body[0].lineno in lines:
                bad.append(f"{path}:{node.body[0].lineno} docstring")
print("\n".join(bad))
sys.exit(1 if bad else 0)
PY
}

check "slugify meets the spec"               python3 -c "from textkit import slugify as s; assert s('  Hello, World! ') == 'hello-world'; assert s('!!!') == ''; assert s('A--B__c') == 'a-b-c'; assert s('Café 2') == 'caf-2'"
check "existing tests still pass"            python3 -m unittest -q
check "tests for slugify were added"         has "test_*.py" "slugify"
check "no comments or docstrings added"      no_new_comments
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
