#!/usr/bin/env python3
"""Check the structure of every skill and eval task. Free and deterministic:
run it before any eval that costs tokens.

Usage: evals/lint.py [repo-root]   (default: this repo)

- Every skill has frontmatter with `name` (its directory) and `description`,
  and a ```contract block with Layer, Input, Output, Stop and Returns filled in.
- Its layer matches its folder: verbs and combinators in kernel/, workflows in workflows/.
- No skill names a repo-relative work/ path: task state lives under $WORK.
- Every writer in KERNEL.md's ```registry block is a skill.
- Every task has prompt.md and checks.sh or rubric.md, and checks.sh comes with
  reference/apply.sh. Its task.md frontmatter covers only real skills, and every
  skill is covered by some task.
"""
import re
import sys
from pathlib import Path

CONTRACT = ["Layer", "Input", "Output", "Stop", "Returns"]
LAYER_DIRS = {"verb": "kernel", "combinator": "kernel", "workflow": "workflows"}


def fields(lines):
    """`Key: value` lines as a dict of stripped values."""
    found = {}
    for line in lines:
        key, sep, value = line.partition(":")
        if sep:
            found[key.strip()] = value.strip()
    return found


def frontmatter(text):
    lines = text.splitlines()
    if lines[:1] != ["---"] or "---" not in lines[1:]:
        return None
    return fields(lines[1:lines.index("---", 1)])


def block(text, info):
    """The lines of the first ```<info> block, or None."""
    match = re.search(rf"^```{info}\n(.*?)^```", text, re.M | re.S)
    return match.group(1).splitlines() if match else None


def lint(root):
    root = Path(root).resolve()
    errors = []

    def err(path, msg):
        errors.append(f"{path.relative_to(root)}: {msg}")

    skills = sorted(root.glob("kernel/*/SKILL.md")) + sorted(root.glob("workflows/*/SKILL.md"))
    names = {p.parent.name for p in skills} | {p.name for p in root.glob(".claude/skills/*")}

    for path in skills:
        text, name, folder = path.read_text(), path.parent.name, path.parent.parent.name
        front = frontmatter(text) or {}
        if front.get("name") != name:
            err(path, f"frontmatter name must be '{name}', its directory")
        if not front.get("description"):
            err(path, "frontmatter has no description")

        lines = block(text, "contract")
        if lines is None:
            err(path, "no ```contract block")
        contract = fields(lines or [])
        for key in CONTRACT:
            if not contract.get(key):
                err(path, f"contract field '{key}' is missing or empty")
        layer = contract.get("Layer")
        if layer and layer not in LAYER_DIRS:
            err(path, f"Layer '{layer}' must be verb, combinator or workflow")
        elif layer and LAYER_DIRS[layer] != folder:
            err(path, f"a {layer} belongs in {LAYER_DIRS[layer]}/, not {folder}/")

        if re.search(r"(^|[^\w/.$-])work/", text, re.M):
            err(path, "names a repo-relative work/ path; task state lives under $WORK/ (KERNEL.md, State)")

    kernel = root / "kernel" / "KERNEL.md"
    if kernel.exists():
        rows = block(kernel.read_text(), "registry")
        if rows is None:
            err(kernel, "no ```registry block")
        for row in rows or []:
            cols = row.split()
            if cols and not cols[0].startswith("#") and (len(cols) < 2 or cols[1] not in names):
                err(kernel, f"registry row '{row.strip()}' needs a writer that is a skill")

    covered = set()
    for task in sorted(p for p in root.glob("evals/tasks/*") if p.is_dir()):
        task_md = task / "task.md"
        front = (frontmatter(task_md.read_text()) if task_md.exists() else None) or {}
        covers = [c.strip() for c in front.get("covers", "").split(",") if c.strip()]
        if not covers:
            err(task, "task.md needs frontmatter naming the skills it covers (covers: a, b)")
        for name in covers:
            if name not in names:
                err(task, f"covers '{name}', which isn't a skill")
        covered.update(covers)
        if not front.get("turns", "1").isdigit():
            err(task, "turns: must be a whole number")
        if not (task / "prompt.md").exists():
            err(task, "no prompt.md")
        if not ((task / "checks.sh").exists() or (task / "rubric.md").exists()):
            err(task, "needs checks.sh or rubric.md")
        if (task / "checks.sh").exists() and not (task / "reference" / "apply.sh").exists():
            err(task, "checks.sh has no reference/apply.sh to test it against (run.py --graders)")

    for path in skills:
        if path.parent.name not in covered:
            err(path, "no eval task covers this skill (covers: in evals/tasks/*/task.md)")
    return errors


if __name__ == "__main__":
    errors = lint(sys.argv[1] if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent)
    print("\n".join(errors + [f"lint: {len(errors)} error(s)" if errors else "lint: ok"]))
    sys.exit(1 if errors else 0)
